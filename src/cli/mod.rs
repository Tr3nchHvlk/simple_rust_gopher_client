use clap::Parser;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLI_ARGS: CliArgs = CliArgs::parse();
}

#[derive(Parser, Debug, Clone)]
pub struct CliArgs {
    /// Specify 
    #[arg(short='H', long="HOST")]
    pub host: String,

    /// Specify the path prefix for which logs file will be saved to. Do not specify if no logs file are to be saved.
    #[arg(short='L', long="LOG_LOC", default_value=None)]
    pub log_path_prefix: Option<String>,

    /// Specify the path prefix for which the folder the files will be downloaded to. Do not specify if download is exempted.
    #[arg(short='D', long="DL_LOC", default_value=None)]
    pub download_path_prefix: Option<String>,

    /// Specify to trace external references. 
    #[arg(long="EXT")]
    pub trace_external: bool,

    /// Specify the default timeout in seconds when establishing connection to a server
    #[arg(long="CONN_TIME", default_value="10")]
    pub conn_timeout: u64,

    /// Specify the default timeout in seconds when accepting response from a server
    #[arg(long="RESP_TIME", default_value="5")]
    pub resp_timeout: u64,

    /// Specify to turn off verbose output in STDOUT
    #[arg(long="DV")]
    pub disable_verbose: bool,
}