use proto_pdk_test_utils::*;

mod composer_tool {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn registers_tool_metadata() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("composer-test").await;
        let output = plugin
            .register_tool(RegisterToolInput {
                id: Id::raw("composer-test"),
                ..Default::default()
            })
            .await;

        assert_eq!(output.name, "Composer");
        assert_eq!(output.type_of, PluginType::DependencyManager);
        assert!(output.minimum_proto_version.is_some());
        assert!(output.plugin_version.is_some());
        assert_eq!(output.requires, vec!["php"]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn loads_versions_from_git() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("composer-test").await;
        let output = plugin.load_versions(LoadVersionsInput::default()).await;

        assert!(!output.versions.is_empty());

        // Should contain Composer 2.x versions
        let version_strings: Vec<String> =
            output.versions.iter().map(|v| v.to_string()).collect();
        assert!(version_strings.iter().any(|v| v.starts_with("2.")));

        // Should NOT contain 1.x versions
        assert!(!version_strings.iter().any(|v| v.starts_with("1.")));
    }

    generate_resolve_versions_tests!("composer-test", {
        "2.8" => "2.8.12",
    });

    #[tokio::test(flavor = "multi_thread")]
    async fn detects_version_files() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("composer-test").await;
        let output = plugin
            .detect_version_files(DetectVersionInput::default())
            .await;

        assert_eq!(output.files, vec!["composer.json"]);
        assert_eq!(output.ignore, vec!["vendor"]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn locates_executables_unix() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("composer-test", |config| {
                config.host(HostOS::Linux, HostArch::X64);
            })
            .await;

        let output = plugin
            .locate_executables(LocateExecutablesInput {
                context: PluginContext {
                    version: VersionSpec::parse("2.8.6").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        let composer = output
            .exes
            .get("composer")
            .expect("composer executable missing");
        assert!(composer.primary);
        assert_eq!(
            composer.exe_path.as_deref(),
            Some(std::path::Path::new("composer"))
        );
        assert_eq!(output.exes_dirs, vec![".".to_string()]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn locates_executables_windows() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("composer-test", |config| {
                config.host(HostOS::Windows, HostArch::X64);
            })
            .await;

        let output = plugin
            .locate_executables(LocateExecutablesInput {
                context: PluginContext {
                    version: VersionSpec::parse("2.8.6").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await;

        let composer = output
            .exes
            .get("composer")
            .expect("composer executable missing");
        assert!(composer.primary);
        assert_eq!(
            composer.exe_path.as_deref(),
            Some(std::path::Path::new("composer.bat"))
        );
    }
}
