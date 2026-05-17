use rspm_core::paths::{RspmHome, sanitize_name};

#[test]
fn home_paths_use_expected_names() {
    let home = RspmHome::new("/tmp/rspm-test");

    assert!(home.rpc_socket().ends_with("rpc.sock"));
    assert!(home.pub_socket().ends_with("pub.sock"));
    assert!(home.dump_file().ends_with("dump.rspm"));
    assert_eq!(sanitize_name("api app"), "api-app");
}
