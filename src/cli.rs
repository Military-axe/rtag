use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rtag", about = "An example of StructOpt usage.")]
pub struct Opt {
    /// 目标tag
    #[structopt(short = "t", long = "tag")]
    pub tag: Vec<String>,

    /// 目标值
    #[structopt(short = "v", long = "value")]
    pub value: Option<String>,
}

pub fn parse_cli() -> Opt {
    let opt = Opt::from_args();
    opt
}
