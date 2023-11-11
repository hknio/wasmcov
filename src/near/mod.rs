mod init;

pub use init::init;

pub fn get_coverage(result: &near_workspaces::result::ViewResultDetails) {
    let coverage = &context.jar_contract.get_coverage().await?;
    let coverage: Vec<u8> = near_sdk::base64::decode(&coverage.logs.last().unwrap()).unwrap();
}
