pub mod archive_tag;
pub mod common;

use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use chrono::Local;
use indexmap::IndexMap;
use toml::Value;
use crate::common::{AcqHeadfileParams, ArchiveParams, DWHeadfileParams, ReconHeadfileParams};

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use crate::Headfile;

    #[test]
    fn serialize_deserialize() {

        // parse headfile, then write it as a new 'test.headfile'
        // reopen both and compare strings for equality

        let h = Headfile::from_file("pfg_cs.headfile").unwrap();
        h.to_file("test.headfile").unwrap();
        let mut f = File::open("pfg_cs.headfile").unwrap();
        let mut s1 = String::new();
        f.read_to_string(&mut s1).unwrap();
        let mut f = File::open("test.headfile").unwrap();
        let mut s2 = String::new();
        f.read_to_string(&mut s2).unwrap();
        assert_eq!(s1, s2);
    }

}



#[derive(Clone,Debug)]
pub struct Headfile {
    /// basic image acq parameters
    acq_params: Option<AcqHeadfileParams>,
    diffusion_params: Option<DWHeadfileParams>,
    recon_params: Option<ReconHeadfileParams>,
    archive_params: Option<ArchiveParams>,
    entries: IndexMap<String,Entry>
}

#[derive(Debug,Clone)]
pub enum Entry {
    /// store a single item
    Scalar(String),
    /// store a matrix of m by n items
    List{m:usize,n:usize,items:Vec<String>},
}


impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Entry::Scalar(s) => write!(f, "{}", s),
            Entry::List{m, n, items} => {
                let items = items.iter().map(|s|{
                    // remove any whitespace within entries
                    s.split_ascii_whitespace().collect::<Vec<_>>().join("")
                }).collect::<Vec<_>>().join(" ");
                write!(f, "{m}:{n},{items}")
            }
        }
    }
}

impl Display for Headfile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;
        let h = self.clone().integrate_params();
        let mut s = String::new();
        for (key,val) in &h.entries {
            writeln!(&mut s,"{key}={val}")?;
        }
        write!(f, "{}", s)
    }
}

impl Headfile {
    pub fn new() -> Headfile {
        Headfile{
            acq_params: None,
            diffusion_params: None,
            recon_params: None,
            archive_params: None,
            entries:IndexMap::new()
        }
    }

    pub fn entries(self) -> IndexMap<String,String> {
        self.integrate_params().entries.into_iter().map(|(k,v)|{
            (k,v.to_string())
        }).collect()
    }


    pub fn from_hash(entries:&IndexMap<String,String>) -> Self {

        let entries:IndexMap<String,Entry> = entries.iter().map(|(key,val)|{
            let entry = Self::parse_entry(val);
            (key.to_string(),entry)
        }).collect();

        Self {
            acq_params: None,
            diffusion_params: None,
            recon_params: None,
            archive_params: None,
            entries,
        }

    }

    pub fn from_file(headfile:impl AsRef<Path>) -> Result<Self, std::io::Error> {

        let mut f = File::open(headfile)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        let entries:IndexMap<String,Entry> = s.lines().map(|line|{
            let (key,val) = line.split_once('=').expect(&format!("did not find = char in line {}",line));
            let key = key.trim();
            let val = val.trim();
            let entry = Self::parse_entry(val);
            (key.to_string(),entry)
        }).collect();

        Ok(
            Self {
                acq_params: None,
                diffusion_params: None,
                recon_params: None,
                archive_params: None,
                entries,
            }
        )

    }

    pub fn project_code(&self) -> Option<String> {
        let code = self.entries.get("U_code")?;
        Some(code.to_string())
    }

    pub fn specimen_id(&self) -> Option<String> {
        let spec = self.entries.get("U_specid")?;
        Some(spec.to_string())
    }

    fn parse_entry(value: &str) -> Entry {
        let value = value.trim();

        if Self::looks_like_list(value) {
            let (dims, rest) = value.split_once(',').unwrap();
            let (m_str, n_str) = dims.split_once(':').unwrap();

            let m = m_str.trim().parse::<usize>().unwrap();
            let n = n_str.trim().parse::<usize>().unwrap();

            let items = rest
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            Entry::List { m, n, items }
        } else {
            Entry::Scalar(value.to_string())
        }
    }

    fn looks_like_list(value: &str) -> bool {
        let value = value.trim();

        let Some((dims, _rest)) = value.split_once(',') else {
            return false;
        };

        let Some((m, n)) = dims.split_once(':') else {
            return false;
        };

        m.trim().parse::<usize>().is_ok() && n.trim().parse::<usize>().is_ok()
    }



    pub fn with_acq_params(mut self, acq_params: AcqHeadfileParams) -> Headfile {
        self.acq_params = Some(acq_params);
        self.integrate_params()
    }

    pub fn with_diffusion_params(mut self, dw_params: DWHeadfileParams) -> Headfile {
        self.diffusion_params = Some(dw_params);
        self.integrate_params()
    }

    pub fn with_recon_params(mut self, reco_params: ReconHeadfileParams) -> Headfile {
        self.recon_params = Some(reco_params);
        self.integrate_params()
    }

    pub fn with_archive_params(mut self, archive_params: ArchiveParams) -> Headfile {
        self.archive_params = Some(archive_params);
        self.integrate_params()
    }

    /// integrates structured parameters into the body of the headfile (index map)
    fn integrate_params(mut self) -> Self {

        let mut h = self.entries.clone();

        if let Some(params) = &self.acq_params {
            let ah = params.to_hash();
            let mut new_entries = ah.into_iter().map(|(key,val)| (key, Entry::Scalar(val)))
                .collect::<IndexMap<String,Entry>>();
            h.append(&mut new_entries);
        }

        if let Some(params) = &self.diffusion_params {
            let ah = params.to_hash();
            let mut new_entries = ah.into_iter().map(|(key,val)| (key, Entry::Scalar(val)))
                .collect::<IndexMap<String,Entry>>();
            h.append(&mut new_entries);
        }

        if let Some(params) = &self.recon_params {
            let ah = params.to_hash();
            let mut new_entries = ah.into_iter().map(|(key,val)| (key, Entry::Scalar(val)))
                .collect::<IndexMap<String,Entry>>();
            h.append(&mut new_entries);
        }

        if let Some(params) = &self.archive_params {
            let ah = params.to_hash();
            let mut new_entries = ah.into_iter().map(|(key,val)| (key, Entry::Scalar(val)))
                .collect::<IndexMap<String,Entry>>();
            h.append(&mut new_entries);
        }

        self.entries = h;
        self

    }


    pub fn to_file(&self,filename:impl AsRef<Path>) -> std::io::Result<()> {
        let hfs = self.to_string();
        let mut f = File::create(filename.as_ref().with_extension("headfile"))?;
        f.write_all(hfs.as_bytes())
    }

    pub fn write_timestamp(&mut self) {
        let now = Local::now();
        let datetime_string = now.format("%Y-%m-%d_%H-%M-%S").to_string();
        self.insert_scalar("hf_timestamp",datetime_string,false);
    }

    pub fn get_string_scalar(&self, entry:&str) -> Option<String> {
        self.entries.get(entry).and_then(|entry|{
            if let Entry::Scalar(val) = entry {
                Some(val.to_string())
            }else {
                None
            }
        })
    }

    pub fn get_numeric_scalar(&self, entry:&str) -> Option<f64> {
        self.entries.get(entry).and_then(|entry|{
            if let Entry::Scalar(val) = entry {
                val.parse::<f64>().ok()
            }else {
                None
            }
        })
    }

    pub fn get_numeric_vector(&self, entry:&str) -> Option<(usize,usize,Vec<f64>)> {
        self.entries.get(entry).and_then(|entry|{
            if let Entry::List {m,n,items} = entry {
                let v:Vec<f64> = items.iter().map(|s| s.parse::<f64>().unwrap()).collect();
                Some((*m,*n,v))
            }else {
                None
            }
        })
    }

    pub fn dim_x(&mut self, dim_x:usize) {
        self.insert_scalar("dim_X",dim_x, false);
    }
    pub fn dim_y(&mut self, dim_y:usize) {
        self.insert_scalar("dim_Y",dim_y, false);
    }
    pub fn dim_z(&mut self, dim_z:usize) {
        self.insert_scalar("dim_Z",dim_z, false);
    }
    pub fn fov_x(&mut self, fov_x:f64) {
        self.insert_scalar("fovx",fov_x, false);
    }
    pub fn fov_y(&mut self, fov_y:f64) {
        self.insert_scalar("fovy",fov_y, false);
    }
    pub fn fov_z(&mut self, fov_z:f64) {
        self.insert_scalar("fovz",fov_z, false);
    }
    pub fn tr(&mut self, tr_us:usize) {
        self.insert_scalar("tr",tr_us, false);
    }
    pub fn te(&mut self, te_ms:f64) {
        self.insert_scalar("te",te_ms, false);
    }
    pub fn bw(&mut self, half_width:f64) {
        self.insert_scalar("bw",half_width, false);
    }
    pub fn ne(&mut self, number_echoes:usize) {
        self.insert_scalar("ne",number_echoes, false);
    }
    pub fn psd_name(&mut self, pulse_seq_name:impl AsRef<str>) {
        self.insert_scalar("S_PSDname",pulse_seq_name.as_ref(), false);
    }
    pub fn bval_dir(&mut self, direction:&[f64]) {
        self.insert_list_1d("bval_dir",direction, false);
    }
    pub fn b_value(&mut self, bval:f64) {
        self.insert_scalar("bvalue",bval, false);
    }
    pub fn n_volumes(&mut self,volumes:usize) {
        self.insert_scalar("volumes",volumes, false);
    }
    pub fn insert_scalar(&mut self, key:&str, item: impl Display, safe:bool) {
        if safe && self.entries.contains_key(key) {
            return
        }
        self.entries.insert(key.to_string(), Entry::Scalar(item.to_string()));
    }

    pub fn insert_list_1d(&mut self, key:&str, items: &[impl Display], safe:bool) {
        if safe && self.entries.contains_key(key) {
            return
        }
        let items = items.iter().map(|item| item.to_string()).collect::<Vec<String>>();
        self.entries.insert(key.to_string(), Entry::List{m:items.len(),n:1,items});
    }

    pub fn insert_list_2d(&mut self, key:&str, m:usize,n:usize, items: &[impl Display], safe:bool) {
        if safe && self.entries.contains_key(key) {
            return
        }
        let items = items.iter().map(|item| item.to_string()).collect::<Vec<String>>();
        self.entries.insert(key.to_string(), Entry::List{m,n,items});
    }

    pub fn insert_toml_table(&mut self, table:&toml::Table, safe_mode:bool) {
        for (key,val) in table {
            match val {
                Value::String(s) => self.insert_scalar(key, s, safe_mode),
                Value::Integer(i) => self.insert_scalar(key, i, safe_mode),
                Value::Float(f) => self.insert_scalar(key, f, safe_mode),
                Value::Boolean(b) => self.insert_scalar(key, b, safe_mode),
                Value::Datetime(d) => self.insert_scalar(key, d, safe_mode),
                Value::Array(a) => self.insert_toml_array(key, a, safe_mode),
                Value::Table(t) => self.insert_toml_table(t, safe_mode),
            }
        }
    }

    fn insert_toml_array(&mut self, key:&str, array:&Vec<Value>, safe_mode:bool) {
        match &array[0] {
            Value::String(_) => {
                let a:Vec<_> = array.iter().map(|val| val.as_str().expect("all values in array must be a string")).collect();
                self.insert_list_1d(key,&a,safe_mode);
            }
            Value::Integer(_) => {
                let a:Vec<_> = array.iter().map(|val| val.as_integer().expect("all values in array must be an integer")).collect();
                self.insert_list_1d(key,&a,safe_mode);
            }
            Value::Float(_) => {
                let a:Vec<_> = array.iter().map(|val| val.as_float().expect("all values in array must be a float")).collect();
                self.insert_list_1d(key,&a,safe_mode);
            }
            Value::Boolean(_) => {
                let a:Vec<_> = array.iter().map(|val| val.as_bool().expect("all values in array must be a boolean")).collect();
                self.insert_list_1d(key,&a,safe_mode);
            }
            Value::Datetime(_) => {
                let a:Vec<_> = array.iter().map(|val| val.as_datetime().expect("all values in array must be a datetime")).collect();
                self.insert_list_1d(key,&a,safe_mode);
            }
            Value::Array(_) => {
                let mut m = 0;
                let n = array.len();
                let entries = array.iter().map(|val| {
                    let a = val.as_array().expect("expected an array of values");
                    m = a.len();
                    a.iter().map(|item| item.to_string()).collect::<Vec<_>>()
                }).flatten().collect::<Vec<String>>();
                self.insert_list_2d(key,m,n,&entries,safe_mode);
                //println!("cannot insert a non-scalar value into an array")
            }
            Value::Table(_) => {
                println!("cannot insert a non-scalar value into an array")
            }
        }
    }

}

pub trait Flatten {
    fn flatten(&self) -> (usize,usize,Vec<impl Display>);
}
