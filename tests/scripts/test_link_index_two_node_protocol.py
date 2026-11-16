"""Offline protocol controls; all filesystem operations are mocked."""
from pathlib import Path
import contextlib
import copy
import io
import json
import sys
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
        record=dict(ino=123,kind=0x8000,mode=0o644,uid=0,gid=0,links=3,
                    size=280,blocks=2,atime=1,mtime=2,ctime=2)
        before=dict(records={p:dict(record) for p in ['/policy','/source','/peer']},
                    flags=[0x81000,0x80000,0x80000],free_blocks=100,free_inodes=50,
                    inventory=[[n.hex(),123] for n in initial])
        for bs,split,debit in [(1024,False,0),(1024,True,1),(4096,True,2)]:
            after=copy.deepcopy(before)
            after['inventory'].append([query.hex(),123]);after['free_blocks']-=debit
            after['records']['/policy'].update(size=280+bs*split,blocks=2+debit*(bs//512),mtime=3,ctime=3)
            after['records']['/source'].update(links=4,ctime=4)
            seen=[];calls=[]
            def audit(root,names,source):
                seen.append((root,names.copy(),source))
                return copy.deepcopy(before if len(seen)==1 else after)
            namespace.update(audit=audit, EXPECTED=[(p.encode(),v) for p,v in before['records'].items()],
                             FLAGS=before['flags'],FREE=(100,50))
            namespace['os']=types.SimpleNamespace(link=lambda a,b:calls.append((a,b)),
                statvfs=lambda root:types.SimpleNamespace(f_flag=0),ST_RDONLY=1)
            out=io.StringIO()
            with contextlib.redirect_stdout(out):
                main(b'/GUEST','link',[n.hex() for n in initial],query.hex(),123,split=split,block_size=bs)
            self.assertEqual(seen,[(b'/GUEST',initial,123),(b'/GUEST',initial+[query],123)])
            self.assertEqual(calls,[(b'/GUEST/source',b'/GUEST/policy/'+query)])
            row=json.loads(out.getvalue());self.assertEqual(row['event'],'link')
            self.assertEqual(row['after'],after);self.assertFalse(row['readonly'])
            # Each corruption violates a different part of the measured delta.
            for path,key,value in [('/policy','mode',0),('/policy','mtime',1),
                                   ('/source','links',5),('/source','ctime',1),('/peer','atime',3)]:
                bad=copy.deepcopy(after);bad['records'][path][key]=value
                with self.assertRaises(AssertionError):namespace['delta'](before,bad,query.hex(),123,split,bs)
            for key,value in [('flags',[0,0,0]),('free_inodes',49),('free_blocks',97),
                              ('inventory',before['inventory'])]:
                bad=copy.deepcopy(after);bad[key]=value
                with self.assertRaises(AssertionError):namespace['delta'](before,bad,query.hex(),123,split,bs)
            namespace.update(audit=lambda *args:after,
                EXPECTED=[(p.encode(),v) for p,v in after['records'].items()],FREE=(100-debit,50))
            namespace['os'].statvfs=lambda root:types.SimpleNamespace(f_flag=1)
            with contextlib.redirect_stdout(io.StringIO()):
                main(b'/GUEST','cold',[n.hex() for n in initial+[query]],query.hex(),123,split=split,block_size=bs)
            # Audit/cold must refuse a writable mount; link must refuse a read-only one.
            with self.assertRaises(AssertionError):
                main(b'/GUEST','link',[n.hex() for n in initial],query.hex(),123,split=split,block_size=bs)
            namespace['os'].statvfs=lambda root:types.SimpleNamespace(f_flag=0)
            with self.assertRaises(AssertionError):
                main(b'/GUEST','cold',[n.hex() for n in initial+[query]],query.hex(),123,split=split,block_size=bs)
    def test_bad_requests_reach_no_filesystem_operation(self):
        def forbidden(*args): raise RuntimeError('invalid request reached filesystem operation')
        namespace['audit']=forbidden
        namespace['os']=types.SimpleNamespace(link=forbidden,statvfs=forbidden)
        for mode,names,query,source in [
            ('link',[cap(1).hex()],cap(1).hex(),123),
            ('other',['61'],cap(1).hex(),123),
            ('link',['61'],'62',123),
            ('link',['61'],cap(1).hex(),0),
            ('link',['61'],cap(1).hex(),True),
        ]:
            with self.subTest(mode=mode,source=source):
                with self.assertRaises(AssertionError):main(b'/GUEST',mode,names,query,source,split=False,block_size=1024)
        for split,bs in [(0,1024),(False,2048)]:
            with self.assertRaises(AssertionError):
                main(b'/GUEST','link',['61'],cap(1).hex(),123,split=split,block_size=bs)

if __name__ == '__main__':
    suite=unittest.defaultTestLoader.loadTestsFromTestCase(Protocol)
    result=unittest.TextTestRunner().run(suite)
    assert result.testsRun == 6 and not result.skipped, 'all six protocol controls must execute'
    sys.exit(not result.wasSuccessful())
