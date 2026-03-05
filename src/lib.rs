use std::fmt::Display;
use indexmap::IndexMap;

#[test]
fn test() {
    let mut h = Headfile::new();
    h.te(2.);
    h.dim_z(100);
    h.bval_dir(&[0.3,1.,0.5]);

    println!("{}", h);
}


pub struct Headfile {
    inner:IndexMap<String,Entry>
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
        let mut s = String::new();
        for (key,val) in &self.inner {
            writeln!(&mut s,"{key}={val}")?;
        }
        write!(f, "{}", s)
    }
}

impl Headfile {
    pub fn new() -> Headfile {
        Headfile{inner:IndexMap::new()}
    }
    pub fn dim_x(&mut self, dim_x:usize) {
        self.insert_scalar("dim_X",dim_x);
    }
    pub fn dim_y(&mut self, dim_y:usize) {
        self.insert_scalar("dim_Y",dim_y);
    }
    pub fn dim_z(&mut self, dim_z:usize) {
        self.insert_scalar("dim_Z",dim_z);
    }
    pub fn fov_x(&mut self, fov_x:f64) {
        self.insert_scalar("fovx",fov_x);
    }
    pub fn fov_y(&mut self, fov_y:f64) {
        self.insert_scalar("fovy",fov_y);
    }
    pub fn fov_z(&mut self, fov_z:f64) {
        self.insert_scalar("fovz",fov_z);
    }
    pub fn tr(&mut self, tr_us:usize) {
        self.insert_scalar("tr",tr_us);
    }
    pub fn te(&mut self, te_ms:f64) {
        self.insert_scalar("te",te_ms);
    }
    pub fn bw(&mut self, half_width:f64) {
        self.insert_scalar("bw",half_width);
    }
    pub fn ne(&mut self, number_echoes:usize) {
        self.insert_scalar("ne",number_echoes);
    }
    pub fn psd_name(&mut self, pulse_seq_name:impl AsRef<str>) {
        self.insert_scalar("S_PSDname",pulse_seq_name.as_ref());
    }
    pub fn bval_dir(&mut self, direction:&[f64]) {
        self.insert_list_1d("bval_dir",direction);
    }
    pub fn b_value(&mut self, bval:f64) {
        self.insert_scalar("bvalue",bval);
    }
    pub fn n_volumes(&mut self,volumes:usize) {
        self.insert_scalar("volumes",volumes);
    }
    pub fn insert_scalar(&mut self, key:&str, item: impl Display) {
        self.inner.insert(key.to_string(), Entry::Scalar(item.to_string()));
    }
    pub fn insert_list_1d(&mut self, key:&str, items: &[impl Display]) {
        let items = items.iter().map(|item| item.to_string()).collect::<Vec<String>>();
        self.inner.insert(key.to_string(), Entry::List{m:items.len(),n:1,items});
    }

    pub fn insert_list_2d(&mut self, key:&str, m:usize,n:usize, items: &[impl Display]) {
        let items = items.iter().map(|item| item.to_string()).collect::<Vec<String>>();
        self.inner.insert(key.to_string(), Entry::List{m,n,items});
    }
}
