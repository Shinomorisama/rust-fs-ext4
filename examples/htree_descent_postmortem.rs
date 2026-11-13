//! Source-bound observation of the preserved 13835 witness; never a repair.
//! Run only through scripts/htree-descent-postmortem.py and the prescribed VM.

#[cfg(unix)]
mod postmortem {
    use fs_ext4::{block_io::FileDevice, dir, file_io, fs::Filesystem, hash, htree, inode::Inode};
    use fs_ext4_test_support::{guest_kernel_read, oracle, sha256_hex};
    use std::{fs, path::Path, process::Output, sync::Arc};

    const IMAGE_SHA: &str = "9d60cf8562f3ef70d8f040701a1e549a9e1f1bb2ff747570707da5074f40992d";
    const NAMES: [&str; 2] = ["a_name_of_twenty_00121", "café_863"];
    const INODES: [u32; 2] = [337, 882];
    const HASHES: [(u32, u32); 2] = [(0xffd2c260, 0xb71eb1ce), (0xffd2c260, 0xe5af7c8f)];
    const GUEST: &str = include_str!("../tests/support/htree_descent_postmortem_guest.py");

    fn digest(path: &Path) -> String {
        sha256_hex(&fs::read(path).expect("read witness bytes"))
    }

    fn save(directory: &Path, label: &str, output: &Output) {
        fs::write(directory.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(directory.join(format!("{label}.stderr")), &output.stderr).unwrap();
        fs::write(
            directory.join(format!("{label}.status")),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
    }

    fn hashes(bytes: &[u8]) -> Result<(), &'static str> {
        let text = std::str::from_utf8(bytes).map_err(|_| "hash answer is not UTF-8")?;
        let lines: Vec<_> = text.lines().filter(|l| l.starts_with("Hash of ")).collect();
        if lines.len() != 2 {
            return Err("expected exactly two independent hash answers");
        }
        for ((name, (major, minor)), line) in NAMES.iter().zip(HASHES).zip(lines) {
            if line != format!("Hash of {name} is 0x{major:x} (minor 0x{minor:x})") {
                return Err("independent hash/name answer differs");
            }
        }
        Ok(())
    }

    fn inode(bytes: &[u8]) -> Result<u32, &'static str> {
        let text = std::str::from_utf8(bytes).map_err(|_| "inode answer is not UTF-8")?;
        let lines: Vec<_> = text
            .lines()
            .filter_map(|l| l.strip_prefix("Inode: "))
            .collect();
        if lines.len() != 1 {
            return Err("expected one independent inode report");
        }
        lines[0]
            .split_whitespace()
            .next()
            .ok_or("missing inode number")?
            .parse()
            .map_err(|_| "invalid inode number")
    }

    fn consistency(image: &str, output: &Path, label: &str) {
        let checked = oracle("e2fsck").args(["-fn", image]).judged();
        save(output, label, &checked.output);
        fs::write(
            output.join(format!("{label}.verdict")),
            format!("{:?}\n", checked.verdict),
        )
        .unwrap();
        checked.clean(label);
    }

    fn observations(image: &Path, output: &Path) {
        let image_string = image.to_str().unwrap();
        consistency(image_string, output, "check-before");
        let requests = NAMES
            .iter()
            .map(|name| {
                format!("dx_hash -h 1 -s 0fca7f62-ba78-5c06-49a1-4ab6381e9270 -- \"{name}\"\n")
            })
            .collect::<String>();
        fs::write(output.join("hash-requests.bin"), &requests).unwrap();
        let answers = oracle("debugfs")
            .args(["-f", "-", image_string])
            .stdin(requests.into_bytes())
            .judged();
        save(output, "hashes", &answers.output);
        let answers = answers.clean("independent witness hashes");
        hashes(&answers.stdout).unwrap();
        for (i, name) in NAMES.iter().enumerate() {
            let report = oracle("debugfs")
                .args(["-R", &format!("stat /bigdir/{name}"), image_string])
                .judged();
            save(output, &format!("inode-{i}"), &report.output);
            let report = report.clean("independent witness inode");
            assert_eq!(inode(&report.stdout).unwrap(), INODES[i]);
        }
        for (label, request) in [
            ("directory-inode", "stat /bigdir"),
            ("htree", "htree_dump /bigdir"),
            ("map-49", "bmap /bigdir 49"),
            ("map-50", "bmap /bigdir 50"),
        ] {
            let report = oracle("debugfs")
                .args(["-R", request, image_string])
                .judged();
            save(output, label, &report.output);
            let report = report.clean(label);
            if label.starts_with("map-") {
                let expected = if label == "map-49" { "4434" } else { "4435" };
                assert_eq!(String::from_utf8_lossy(&report.stdout).trim(), expected);
            } else if label == "directory-inode" {
                assert_eq!(inode(&report.stdout).unwrap(), 12);
            }
        }

        cold_lookups(image_string, output);
        consistency(image_string, output, "check-after");
        driver_observations(image_string, output);
    }

    fn driver_observations(image_string: &str, output: &Path) {
        // These are current-driver observations, distinct from the VM oracles.
        // Preserve the original single-leaf miss while measuring public lookup.
        let driver = Filesystem::mount(Arc::new(FileDevice::open(image_string).unwrap())).unwrap();
        let raw = Inode::parse(&driver.read_inode_raw(12).unwrap()).unwrap();
        let data = file_io::read_all(&driver, &raw).unwrap();
        let blocks: Vec<_> = data.chunks(driver.sb.block_size() as usize).collect();
        let mut records = String::new();
        for (i, name) in NAMES.iter().enumerate() {
            let selected = htree::lookup_leaf_with(
                name.as_bytes(),
                blocks[0],
                &driver.sb.hash_seed,
                driver.sb.unsigned_hash(),
                |logical| Ok(blocks[logical as usize].to_vec()),
            )
            .unwrap();
            let actual: Vec<_> = blocks
                .iter()
                .enumerate()
                .skip(1)
                .filter_map(|(logical, bytes)| {
                    dir::parse_block(bytes, true)
                        .unwrap()
                        .iter()
                        .any(|e| e.name == name.as_bytes())
                        .then_some(logical)
                })
                .collect();
            let h = hash::name_hash(
                name.as_bytes(),
                hash::HashVersion::from_u8(1).unwrap(),
                &driver.sb.hash_seed,
            );
            let public = driver
                .lookup_path_bytes(format!("/bigdir/{name}").as_bytes())
                .unwrap();
            records.push_str(&format!(
                "{}|{:08x}|{:08x}|{selected:?}|{actual:?}|{public}\n",
                hex::encode(name),
                h.major,
                h.minor
            ));
            fs::write(output.join("driver-observations.txt"), &records).unwrap();
            assert_eq!(selected, Some(49));
            assert_eq!(actual, vec![49 + i]);
            assert_eq!(public, INODES[i]);
        }
        drop(driver);
    }

    fn cold_lookups(image_string: &str, output: &Path) {
        for order in 0..2 {
            let script =
                format!("python3 - \"$MNT\" {order} <<'POSTMORTEM_PY'\n{GUEST}\nPOSTMORTEM_PY\n");
            let found = guest_kernel_read(image_string, &script);
            save(output, &format!("cold-lookup-{order}"), &found);
            assert!(
                found.status.success(),
                "cold Linux lookup failed; see retained streams"
            );
            let text = std::str::from_utf8(&found.stdout).unwrap();
            let lines: Vec<_> = text.lines().collect();
            assert_eq!(lines.len(), 5);
            assert_eq!(lines[0], "readonly|1");
            for position in 0..2 {
                let i = if order == 0 { position } else { 1 - position };
                assert_eq!(
                    lines[position + 1],
                    format!("{}|{}|0", hex::encode(NAMES[i]), INODES[i])
                );
            }
            assert_eq!(lines[3], "missing|ENOENT");
            assert!(lines[4].starts_with("kernel|") && lines[4].len() > 7);
        }
    }

    pub fn main() {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        assert_eq!(
            args.len(),
            2,
            "usage: htree_descent_postmortem INPUT NEW_OUTPUT"
        );
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap();
        let input = Path::new(&args[0]).canonicalize().unwrap();
        let output = Path::new(&args[1]);
        assert!(input.starts_with(&repo));
        assert!(output
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&repo));
        assert_eq!(digest(&input), IMAGE_SHA);
        fs::create_dir(output).expect("preserve any existing postmortem output");
        let private = output.join("private-check.img");
        fs::copy(&input, &private).unwrap();
        assert_eq!(digest(&private), IMAGE_SHA);
        let result = std::panic::catch_unwind(|| observations(&private, output));
        let original_sha = digest(&input);
        let private_sha = digest(&private);
        fs::write(
            output.join("final-integrity.txt"),
            format!("input {original_sha}\nprivate {private_sha}\n"),
        )
        .unwrap();
        assert_eq!(original_sha, IMAGE_SHA, "immutable witness changed");
        assert_eq!(
            private_sha, IMAGE_SHA,
            "private image changed during read-only checks"
        );
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        fs::write(
            output.join("completion.txt"),
            "independent checks complete\n",
        )
        .unwrap();
        println!("postmortem: two independent hashes/inodes; two cold read-only lookup orders; before/after clean; image bytes unchanged");
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        const ANSWERS: &str = "Hash of a_name_of_twenty_00121 is 0xffd2c260 (minor 0xb71eb1ce)\nHash of café_863 is 0xffd2c260 (minor 0xe5af7c8f)\n";

        #[test]
        fn exact_independent_hashes_are_accepted() {
            assert!(hashes(ANSWERS.as_bytes()).is_ok());
        }
        #[test]
        fn absent_duplicate_and_extra_hash_answers_are_refused() {
            for text in [
                String::new(),
                ANSWERS.lines().next().unwrap().into(),
                ANSWERS.repeat(2),
            ] {
                assert!(hashes(text.as_bytes()).is_err());
            }
        }
        #[test]
        fn changed_name_or_either_hash_component_is_refused() {
            for text in [
                ANSWERS.replace("café_863", "cafe_863"),
                ANSWERS.replace("ffd2c260", "ffd2c262"),
                ANSWERS.replace("e5af7c8f", "e5af7c8e"),
            ] {
                assert!(hashes(text.as_bytes()).is_err());
            }
        }
        #[test]
        fn malformed_hash_reports_are_refused() {
            for text in [
                ANSWERS.replace("0xffd2c260", "not-a-hash"),
                ANSWERS.replace("minor", "other"),
                ANSWERS.replace(')', ""),
            ] {
                assert!(hashes(text.as_bytes()).is_err());
            }
            assert!(hashes(&[0xff]).is_err());
        }
        #[test]
        fn one_independent_inode_report_is_accepted() {
            assert_eq!(inode(b"Inode: 882   Type: regular\nSize: 0\n"), Ok(882));
        }
        #[test]
        fn missing_duplicate_or_malformed_inode_reports_are_refused() {
            for text in [
                "",
                "Inode: \n",
                "Inode: broken\n",
                "Inode: 337\nInode: 882\n",
            ] {
                assert!(inode(text.as_bytes()).is_err());
            }
        }
    }
}

#[cfg(unix)]
fn main() {
    postmortem::main();
}

#[cfg(not(unix))]
fn main() {
    panic!("this postmortem requires the Unix fs-linux-test-harness workflow");
}
