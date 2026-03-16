use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ArchiveTag {
    pub runno: String,
    pub civm_id: String,
    pub archive_engine_base_dir: PathBuf,
    pub n_raw_files: usize,
    pub project_code: String,
    pub raw_file_ext: String,
}

impl ArchiveTag {
    fn name_ready(&self) -> String {
        format!("READY_{}", self.runno)
    }
    pub fn to_file(&self, location: impl AsRef<Path>) {
        let base_dir = self.archive_engine_base_dir.to_str().unwrap();
        let txt = vec![
            format!(
                "{},{},{},{},.{}",
                self.runno, base_dir, self.n_raw_files, self.project_code, self.raw_file_ext
            ),
            format!("# recon_person={}", self.civm_id),
            format!("# tag_file_creator=Wyatt_rust\n"),
        ]
            .join("\n");
        let fp = self.filepath(location);
        if let Ok(mut f) = File::create(&fp) {
            f.write_all(txt.as_bytes()).expect("Unable to write data");
        }else {
            panic!("failed to create archive tag: {}",fp.display());
        }
    }
    pub fn filepath(&self, location: impl AsRef<Path>) -> PathBuf {
        location.as_ref().join(self.name_ready())
    }
}