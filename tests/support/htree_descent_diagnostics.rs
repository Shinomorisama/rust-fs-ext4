//! Retained observations only; the descent test's verdict stays with its caller.
use fs_ext4::{dir, fs::Filesystem, hash, htree, inode::Inode};
use fs_ext4_test_support::{create_unique_temp_dir, oracle, sha256_hex};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Output;

pub struct Evidence {
    base: PathBuf,
    working: PathBuf,
    queries: BufWriter<std::fs::File>,
}

pub struct Inventory {
    pub version: hash::HashVersion,
    pub locations: HashMap<Vec<u8>, Vec<u32>>,
}

impl Evidence {
    pub fn new(image: &str, algorithm: &str, flags: u32, names: &[Vec<u8>]) -> Self {
        // scripts/test.sh deletes its owned scratch directory even on failure.
        // This separate repository-local directory survives that cleanup and is
        // always uploaded by the focused/native/guest CI paths when reached.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tmp/htree-descent-evidence")
            .join(format!("{algorithm}-{flags}"));
        let base = create_unique_temp_dir(&root).unwrap();
        std::fs::write(
            base.join("profile.json"),
            format!(
                "{{\"algorithm\":\"{algorithm}\",\"flags\":{flags},\"names\":{},\"working_path_hex\":\"{}\"}}\n",
                names.len(),
                hex::encode(image.as_bytes())
            ),
        )
        .unwrap();
        let mut fixture = String::new();
        for name in names {
            writeln!(fixture, "{}", hex::encode(name)).unwrap();
        }
        std::fs::write(base.join("names.hex"), fixture).unwrap();
        let queries = BufWriter::new(std::fs::File::create(base.join("queries.jsonl")).unwrap());
        Self {
            base,
            working: image.into(),
            queries,
        }
    }

    pub fn output(&self, label: &str, out: &Output) {
        std::fs::write(self.base.join(format!("{label}.stdout")), &out.stdout).unwrap();
        std::fs::write(self.base.join(format!("{label}.stderr")), &out.stderr).unwrap();
        std::fs::write(
            self.base.join(format!("{label}.status")),
            format!("{:?}\n", out.status.code()),
        )
        .unwrap();
    }

    pub fn snapshot(&self, phase: &str) {
        let image = self.base.join(format!("{phase}.img"));
        std::fs::copy(&self.working, &image).unwrap();
        let digest = sha256_hex(&std::fs::read(image).unwrap());
        std::fs::write(self.base.join(format!("{phase}.sha256")), digest).unwrap();
    }

    pub fn reports(&self) {
        for (label, request) in [
            ("stats", "stats"),
            ("inode", "stat /bigdir"),
            ("htree", "htree_dump /bigdir"),
        ] {
            let out = oracle("debugfs")
                .args(["-R", request])
                .arg(&self.working)
                .output();
            self.output(label, &out);
            assert!(out.status.success(), "retained {label}: {:?}", self.base);
        }
        let check = oracle("e2fsck").arg("-fn").arg(&self.working).judged();
        self.output("check", &check.output);
        std::fs::write(
            self.base.join("check.verdict"),
            format!("{:?}", check.verdict),
        )
        .unwrap();
        check.clean("retained indexed descent input");
    }

    pub fn directory(
        &self,
        fs: &Filesystem,
        ino: u32,
        inode: &Inode,
        data: &[u8],
        names: &[Vec<u8>],
    ) -> Inventory {
        let bs = fs.sb.block_size() as usize;
        let blocks: Vec<_> = data.chunks(bs).collect();
        std::fs::write(self.base.join("directory.bin"), data).unwrap();
        std::fs::write(
            self.base.join("directory-inode.bin"),
            fs.read_inode_raw(ino).unwrap(),
        )
        .unwrap();
        let info = htree::parse_root_info(blocks[0]).unwrap();
        let version = hash::effective_version(info.hash_version, fs.sb.unsigned_hash());
        let seed: Vec<_> = fs
            .sb
            .hash_seed
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let seed = hex::encode(seed);
        let uuid = format!(
            "{}-{}-{}-{}-{}",
            &seed[..8],
            &seed[8..12],
            &seed[12..16],
            &seed[16..20],
            &seed[20..]
        );
        std::fs::write(
            self.base.join("filesystem.json"),
            format!(
                "{{\"uuid_hex\":\"{}\",\"seed_hex\":\"{seed}\",\"seed_uuid\":\"{uuid}\",\"version\":{},\"unsigned\":{},\"depth\":{},\"block_size\":{bs},\"inode\":{ino}}}\n",
                hex::encode(fs.sb.uuid), version as u8, fs.sb.unsigned_hash(), info.indirect_levels
            ),
        )
        .unwrap();
        let (_, roots) = htree::parse_root_entries(blocks[0]).unwrap();
        let mut index = HashSet::from([0]);
        let mut routes = String::new();
        for entry in roots {
            writeln!(
                routes,
                "{{\"parent\":0,\"hash\":{},\"logical\":{}}}",
                entry.hash, entry.block
            )
            .unwrap();
            if info.indirect_levels == 1 {
                index.insert(entry.block);
                let (_, children) =
                    htree::parse_node_entries(blocks[entry.block as usize]).unwrap();
                for child in children {
                    writeln!(
                        routes,
                        "{{\"parent\":{},\"hash\":{},\"logical\":{}}}",
                        entry.block, child.hash, child.block
                    )
                    .unwrap();
                }
            }
        }
        std::fs::write(self.base.join("routes.jsonl"), routes).unwrap();
        let mut entries = String::new();
        let mut maps = String::new();
        let mut locations: HashMap<Vec<u8>, Vec<u32>> = HashMap::new();
        for (logical, block) in blocks.iter().enumerate() {
            writeln!(
                maps,
                "{{\"logical\":{logical},\"physical\":{:?}}}",
                fs.map_inode_logical(inode, logical as u64)
                    .unwrap()
                    .unwrap()
            )
            .unwrap();
            if index.contains(&(logical as u32)) {
                continue;
            }
            match dir::parse_block(block, true) {
                Ok(records) => {
                    for entry in records {
                        let h = hash::name_hash(&entry.name, version, &fs.sb.hash_seed);
                        writeln!(entries, "{{\"logical\":{logical},\"name_hex\":\"{}\",\"inode\":{},\"major\":{},\"minor\":{}}}", hex::encode(&entry.name), entry.inode, h.major, h.minor).unwrap();
                        locations
                            .entry(entry.name)
                            .or_default()
                            .push(logical as u32);
                    }
                }
                Err(error) => {
                    writeln!(
                        entries,
                        "{{\"logical\":{logical},\"parse_error_hex\":\"{}\"}}",
                        hex::encode(format!("{error:?}"))
                    )
                    .unwrap();
                }
            }
        }
        std::fs::write(self.base.join("entries.jsonl"), entries).unwrap();
        std::fs::write(self.base.join("mapping.jsonl"), maps).unwrap();

        // Keep the raw batch and tool answers; do not make our hashes the
        // diagnostic oracle or change the caller's selected-leaf assertion.
        let mut requests = Vec::new();
        for name in names {
            assert!(!name
                .iter()
                .any(|b| matches!(b, 0 | b'\n' | b'\r' | b'"' | b'\\' | b'/')));
            requests.extend_from_slice(
                format!("dx_hash -h {} -s {uuid} -- \"", version as u8).as_bytes(),
            );
            requests.extend_from_slice(name);
            requests.extend_from_slice(b"\"\n");
        }
        std::fs::write(self.base.join("hash-requests.bin"), &requests).unwrap();
        let out = oracle("debugfs")
            .args(["-f", "-"])
            .arg(&self.working)
            .stdin(requests)
            .output();
        self.output("hashes", &out);
        assert!(out.status.success());
        assert_eq!(
            out.stdout
                .split(|b| *b == b'\n')
                .filter(|s| s.starts_with(b"Hash of "))
                .count(),
            names.len()
        );
        Inventory { version, locations }
    }

    pub fn query(
        &mut self,
        name: &[u8],
        h: hash::NameHash,
        leaf: u32,
        found: bool,
        actual: &[u32],
    ) {
        writeln!(self.queries, "{{\"name_hex\":\"{}\",\"major\":{},\"minor\":{},\"selected\":{leaf},\"found\":{found},\"actual\":{actual:?}}}", hex::encode(name), h.major, h.minor).unwrap();
    }

    pub fn complete(&mut self, wrong: usize) {
        self.queries.flush().unwrap();
        std::fs::write(
            self.base.join("result.json"),
            format!("{{\"wrong\":{wrong}}}\n"),
        )
        .unwrap();
    }
}

impl Drop for Evidence {
    fn drop(&mut self) {
        let _ = self.queries.flush();
        if std::thread::panicking() && self.working.is_file() {
            if let Err(error) =
                std::fs::copy(&self.working, self.base.join("working-at-failure.img"))
            {
                let _ = std::fs::write(self.base.join("capture-error.txt"), error.to_string());
            }
        }
    }
}
