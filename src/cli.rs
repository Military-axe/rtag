use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rtag", about = "rtag [option] [value]")]
pub struct Opt {
    /// 目标tag
    #[structopt(short = "t", long = "tag")]
    pub tag: Vec<String>,

    /// 目标值
    #[structopt(short = "v", long = "value")]
    pub value: Option<String>,

    /// 导出
    #[structopt(short = "e", long = "export")]
    pub export: Option<String>,

    /// 导入
    #[structopt(short = "i", long = "import")]
    pub import: Option<String>,
}

pub fn parse_cli() -> Opt {
    let opt = Opt::from_args();
    opt
}
