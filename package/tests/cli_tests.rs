//! CLI integration tests for the buckos package manager
//!
//! These tests verify that the CLI argument parsing works correctly
//! and that commands produce expected outputs.

use std::process::Command;

/// Helper to run buckos CLI commands
fn run_buckos(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", "buckos", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command")
}

/// Helper to check if output contains expected text
fn output_contains(output: &std::process::Output, text: &str) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout.contains(text) || stderr.contains(text)
}

mod cli_parsing {
    use super::*;

    #[test]
    fn test_help_flag() {
        let output = run_buckos(&["--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Buckos Package Manager"));
        assert!(output_contains(&output, "USAGE"));
    }

    #[test]
    fn test_version_flag() {
        let output = run_buckos(&["--version"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "buckos"));
    }

    #[test]
    fn test_install_help() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Install packages"));
    }

    #[test]
    fn test_remove_help() {
        let output = run_buckos(&["remove", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Remove"));
    }

    #[test]
    fn test_update_help() {
        let output = run_buckos(&["update", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Update"));
    }

    #[test]
    fn test_sync_help() {
        let output = run_buckos(&["sync", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Sync"));
    }

    #[test]
    fn test_search_help() {
        let output = run_buckos(&["search", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Search"));
    }

    #[test]
    fn test_info_help() {
        let output = run_buckos(&["info", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "package"));
    }

    #[test]
    fn test_list_help() {
        let output = run_buckos(&["list", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "List"));
    }

    #[test]
    fn test_build_help() {
        let output = run_buckos(&["build", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Build"));
    }

    #[test]
    fn test_clean_help() {
        let output = run_buckos(&["clean", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Clean"));
    }

    #[test]
    fn test_verify_help() {
        let output = run_buckos(&["verify", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_query_help() {
        let output = run_buckos(&["query", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Query"));
    }

    #[test]
    fn test_depclean_help() {
        let output = run_buckos(&["depclean", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Remove unused"));
    }

    #[test]
    fn test_audit_help() {
        let output = run_buckos(&["audit", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_help() {
        let output = run_buckos(&["useflags", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "USE"));
    }

    #[test]
    fn test_detect_help() {
        let output = run_buckos(&["detect", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Detect"));
    }

    #[test]
    fn test_configure_help() {
        let output = run_buckos(&["configure", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "configuration"));
    }

    #[test]
    fn test_set_help() {
        let output = run_buckos(&["set", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "set"));
    }

    #[test]
    fn test_deps_help() {
        let output = run_buckos(&["deps", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "dependencies"));
    }

    #[test]
    fn test_rdeps_help() {
        let output = run_buckos(&["rdeps", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "reverse"));
    }

    #[test]
    fn test_profile_help() {
        let output = run_buckos(&["profile", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "profile"));
    }

    #[test]
    fn test_export_help() {
        let output = run_buckos(&["export", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Export"));
    }

    #[test]
    fn test_revdep_help() {
        let output = run_buckos(&["revdep", "--help"]);
        assert!(output.status.success());
        assert!(output_contains(&output, "Rebuild"));
    }

    #[test]
    fn test_unmerge_alias() {
        let output = run_buckos(&["unmerge", "--help"]);
        assert!(output.status.success());
        // Should work as alias for remove
    }

    #[test]
    fn test_use_alias() {
        let output = run_buckos(&["use", "--help"]);
        assert!(output.status.success());
        // Should work as alias for useflags
    }
}

mod global_options {
    use super::*;

    #[test]
    fn test_verbose_flag() {
        let output = run_buckos(&["-v", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_quiet_flag() {
        let output = run_buckos(&["-q", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_pretend_flag() {
        let output = run_buckos(&["-p", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_ask_flag() {
        let output = run_buckos(&["-a", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_fetchonly_flag() {
        let output = run_buckos(&["--fetchonly", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_oneshot_flag() {
        let output = run_buckos(&["--oneshot", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_oneshot_short_flag() {
        let output = run_buckos(&["-1", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_deep_flag() {
        let output = run_buckos(&["--deep", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_deep_short_flag() {
        let output = run_buckos(&["-D", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_newuse_flag() {
        let output = run_buckos(&["--newuse", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_newuse_short_flag() {
        let output = run_buckos(&["-N", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_tree_flag() {
        let output = run_buckos(&["--tree", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_jobs_flag() {
        let output = run_buckos(&["-j", "4", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_config_flag() {
        let output = run_buckos(&["-c", "/tmp/config.toml", "--help"]);
        assert!(output.status.success());
    }
}

mod install_options {
    use super::*;

    #[test]
    fn test_install_force_flag() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "force"));
    }

    #[test]
    fn test_install_nodeps_flag() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "nodeps"));
    }

    #[test]
    fn test_install_build_flag() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "build"));
    }

    #[test]
    fn test_install_use_flags() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "use"));
    }

    #[test]
    fn test_install_noreplace_flag() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "noreplace"));
    }

    #[test]
    fn test_install_emptytree_flag() {
        let output = run_buckos(&["install", "--help"]);
        assert!(output_contains(&output, "emptytree"));
    }
}

mod remove_options {
    use super::*;

    #[test]
    fn test_remove_force_flag() {
        let output = run_buckos(&["remove", "--help"]);
        assert!(output_contains(&output, "force"));
    }

    #[test]
    fn test_remove_recursive_flag() {
        let output = run_buckos(&["remove", "--help"]);
        assert!(output_contains(&output, "recursive"));
    }
}

mod update_options {
    use super::*;

    #[test]
    fn test_update_nosync_flag() {
        let output = run_buckos(&["update", "--help"]);
        assert!(output_contains(&output, "nosync"));
    }

    #[test]
    fn test_update_check_flag() {
        let output = run_buckos(&["update", "--help"]);
        assert!(output_contains(&output, "check"));
    }

    #[test]
    fn test_update_with_bdeps_flag() {
        let output = run_buckos(&["update", "--help"]);
        assert!(output_contains(&output, "bdeps"));
    }
}

mod sync_options {
    use super::*;

    #[test]
    fn test_sync_all_flag() {
        let output = run_buckos(&["sync", "--help"]);
        assert!(output_contains(&output, "all"));
    }

    #[test]
    fn test_sync_webrsync_flag() {
        let output = run_buckos(&["sync", "--help"]);
        assert!(output_contains(&output, "webrsync"));
    }
}

mod query_subcommands {
    use super::*;

    #[test]
    fn test_query_files_help() {
        let output = run_buckos(&["query", "files", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_query_deps_help() {
        let output = run_buckos(&["query", "deps", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_query_rdeps_help() {
        let output = run_buckos(&["query", "rdeps", "--help"]);
        assert!(output.status.success());
    }
}

mod useflags_subcommands {
    use super::*;

    #[test]
    fn test_useflags_list_help() {
        let output = run_buckos(&["useflags", "list", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_info_help() {
        let output = run_buckos(&["useflags", "info", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_set_help() {
        let output = run_buckos(&["useflags", "set", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_get_help() {
        let output = run_buckos(&["useflags", "get", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_package_help() {
        let output = run_buckos(&["useflags", "package", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_expand_help() {
        let output = run_buckos(&["useflags", "expand", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_useflags_validate_help() {
        let output = run_buckos(&["useflags", "validate", "--help"]);
        assert!(output.status.success());
    }
}

mod set_subcommands {
    use super::*;

    #[test]
    fn test_set_list_help() {
        let output = run_buckos(&["set", "list", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_set_show_help() {
        let output = run_buckos(&["set", "show", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_set_install_help() {
        let output = run_buckos(&["set", "install", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_set_compare_help() {
        let output = run_buckos(&["set", "compare", "--help"]);
        assert!(output.status.success());
    }
}

mod patch_subcommands {
    use super::*;

    #[test]
    fn test_patch_list_help() {
        let output = run_buckos(&["patch", "list", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_patch_info_help() {
        let output = run_buckos(&["patch", "info", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_patch_add_help() {
        let output = run_buckos(&["patch", "add", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_patch_remove_help() {
        let output = run_buckos(&["patch", "remove", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_patch_check_help() {
        let output = run_buckos(&["patch", "check", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_patch_order_help() {
        let output = run_buckos(&["patch", "order", "--help"]);
        assert!(output.status.success());
    }
}

mod profile_subcommands {
    use super::*;

    #[test]
    fn test_profile_list_help() {
        let output = run_buckos(&["profile", "list", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_profile_show_help() {
        let output = run_buckos(&["profile", "show", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_profile_set_help() {
        let output = run_buckos(&["profile", "set", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_profile_current_help() {
        let output = run_buckos(&["profile", "current", "--help"]);
        assert!(output.status.success());
    }
}

mod invalid_commands {
    use super::*;

    #[test]
    fn test_invalid_command() {
        let output = run_buckos(&["invalid-command"]);
        assert!(!output.status.success());
    }

    #[test]
    fn test_install_without_packages() {
        let output = run_buckos(&["install"]);
        assert!(!output.status.success());
        assert!(output_contains(&output, "required"));
    }

    #[test]
    fn test_remove_without_packages() {
        let output = run_buckos(&["remove"]);
        assert!(!output.status.success());
        assert!(output_contains(&output, "required"));
    }

    #[test]
    fn test_search_without_query() {
        let output = run_buckos(&["search"]);
        assert!(!output.status.success());
    }

    #[test]
    fn test_info_without_package() {
        let output = run_buckos(&["info"]);
        assert!(!output.status.success());
    }

    #[test]
    fn test_build_without_target() {
        let output = run_buckos(&["build"]);
        assert!(!output.status.success());
    }

    #[test]
    fn test_owner_without_path() {
        let output = run_buckos(&["owner"]);
        assert!(!output.status.success());
    }

    #[test]
    fn test_depgraph_without_package() {
        let output = run_buckos(&["depgraph"]);
        assert!(!output.status.success());
    }
}

mod combined_flags {
    use super::*;

    #[test]
    fn test_multiple_verbose_flags() {
        let output = run_buckos(&["-vvv", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_pretend_and_ask() {
        let output = run_buckos(&["-p", "-a", "--help"]);
        assert!(output.status.success());
    }

    #[test]
    fn test_deep_and_newuse() {
        let output = run_buckos(&["-D", "-N", "--help"]);
        assert!(output.status.success());
    }
}
