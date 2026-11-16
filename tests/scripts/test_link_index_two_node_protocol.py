"""Offline protocol controls; all filesystem operations are mocked."""
from pathlib import Path
import contextlib
import io
import json
import types
import unittest

path = Path(__file__).resolve().parents[2] / 'src/fs/guarded_chmod/needs_host/link_index_two_node_guest.py'
namespace={'__name__':'protocol_check'}
exec(compile(path.read_text(),str(path),'exec'),namespace)
component=namespace['component']; decode=namespace['namespace']; main=namespace['main']
cap=lambda n: (f'cap-{n:05d}-'.encode()+b'n'*245)

class Protocol(unittest.TestCase):
    def test_non_dense_names_and_raw_bytes_are_preserved(self):
        wanted=sorted([cap(1),cap(99),b'raw-\xff'])
        self.assertEqual(decode([n.hex() for n in wanted]),wanted)
    def test_bad_components_are_rejected(self):
        for value in ['', '2e','2e2e','2f','00','a', 'XX', 'FF','66 66', '61'*256, 42]:
            with self.subTest(value=str(value)[:16]):
                with self.assertRaises((AssertionError,ValueError)):
                    component(value)
    def test_bad_namespaces_are_rejected(self):
        for values in [[],['62','61'],['61','61'],{},['61']*16385]:
            with self.subTest(size=len(values)):
                with self.assertRaises(AssertionError): decode(values)
    def test_namespace_limit_is_exact(self):
        values=[f'n{n:05d}'.encode().hex() for n in range(16384)]
        self.assertEqual(len(decode(values)),16384)
        with self.assertRaises(AssertionError):decode(values+[b'z'.hex()])
    def test_link_uses_only_the_explicit_namespace(self):
        initial=[cap(1),cap(99)]; query=cap(120)
        seen=[]; calls=[]
        def audit(root,names,source):
            seen.append((root,names.copy(),source))
            return {'inventory':[[n.hex(),source] for n in names]}
        namespace['audit']=audit
        namespace['os']=types.SimpleNamespace(link=lambda a,b:calls.append((a,b)))
        out=io.StringIO()
        with contextlib.redirect_stdout(out):
            main(b'/GUEST','link',[n.hex() for n in initial],query.hex(),123)
        self.assertEqual(seen,[(b'/GUEST',initial,123),(b'/GUEST',initial+[query],123)])
        self.assertEqual(calls,[(b'/GUEST/source',b'/GUEST/policy/'+query)])
        row=json.loads(out.getvalue());self.assertEqual(row['event'],'link')
        self.assertEqual(row['after']['inventory'],[[n.hex(),123] for n in initial+[query]])
    def test_bad_requests_reach_no_filesystem_operation(self):
        def forbidden(*args): raise RuntimeError('invalid request reached filesystem operation')
        namespace['audit']=forbidden
        namespace['os']=types.SimpleNamespace(link=forbidden)
        for mode,names,query,source in [
            ('link',[cap(1).hex()],cap(1).hex(),123),
            ('other',['61'],cap(1).hex(),123),
            ('link',['61'],'62',123),
            ('link',['61'],cap(1).hex(),0),
            ('link',['61'],cap(1).hex(),True),
        ]:
            with self.subTest(mode=mode,source=source):
                with self.assertRaises(AssertionError):main(b'/GUEST',mode,names,query,source)

if __name__ == '__main__':
    unittest.main()
