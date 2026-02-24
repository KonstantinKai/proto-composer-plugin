use crate::config::ComposerPluginConfig;
use extism_pdk::*;
use proto_pdk::*;
use std::collections::HashMap;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

static NAME: &str = "Composer";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::DependencyManager,
        minimum_proto_version: Some(Version::new(0, 46, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        requires: vec!["php".into()],
        ..RegisterToolOutput::default()
    }))
}

#[plugin_fn]
pub fn define_tool_config(_: ()) -> FnResult<Json<DefineToolConfigOutput>> {
    Ok(Json(DefineToolConfigOutput {
        schema: schematic::SchemaBuilder::build_root::<ComposerPluginConfig>(),
    }))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec!["composer.json".into()],
        ignore: vec!["vendor".into()],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(_input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let version = None;

    // Composer version pinning via composer.json is uncommon.
    // Could parse require.composer-plugin-api in the future.
    // For now, this is a no-op — version comes from .prototools or CLI.

    Ok(Json(ParseVersionFileOutput { version }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let config = get_tool_config::<ComposerPluginConfig>()?;
    let allow_pre = config.allow_pre_releases;
    let tags = load_git_tags("https://github.com/composer/composer")?;

    // Composer tags: 2.8.6, 2.7.0, 1.10.27, etc.
    let versions: Vec<String> = tags
        .into_iter()
        .filter_map(|tag| {
            // Only Composer 2.x+ (skip 1.x)
            if !tag.starts_with('2') {
                return None;
            }
            // Skip pre-release versions unless opted in
            if !allow_pre {
                let lower = tag.to_lowercase();
                if lower.contains("rc") || lower.contains("alpha") || lower.contains("beta") {
                    return None;
                }
            }
            Some(tag)
        })
        .collect();

    Ok(Json(LoadVersionsOutput::from(versions)?))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    if let UnresolvedVersionSpec::Alias(alias) = input.initial {
        match alias.as_str() {
            "lts" | "stable" => {
                output.candidate = Some(UnresolvedVersionSpec::parse("latest")?);
            }
            _ => {}
        }
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    let env = get_host_environment()?;
    let version = &input.context.version;
    let install_dir = input
        .install_dir
        .real_path()
        .unwrap_or_else(|| input.install_dir.to_path_buf());

    let phar_url = format!("https://getcomposer.org/download/{version}/composer.phar");

    if env.os == HostOS::Windows {
        // Download composer.phar
        let phar_path = format!("{}\\composer.phar", install_dir.display());

        let download = exec_command!(
            pipe,
            "powershell",
            [
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                    phar_url, phar_path,
                ),
            ]
        );

        if download.exit_code != 0 {
            return Ok(Json(NativeInstallOutput {
                error: Some(format!(
                    "Failed to download composer.phar: {}",
                    download.stderr
                )),
                installed: false,
                ..NativeInstallOutput::default()
            }));
        }

        // Create composer.bat wrapper
        let bat_path = format!("{}\\composer.bat", install_dir.display());
        let bat_content = r#"@php "%~dp0composer.phar" %*"#;

        let write_bat = exec_command!(
            pipe,
            "cmd",
            ["/c", &format!("echo {bat_content}> \"{bat_path}\""),]
        );

        if write_bat.exit_code != 0 {
            return Ok(Json(NativeInstallOutput {
                error: Some(format!(
                    "Failed to create composer.bat: {}",
                    write_bat.stderr
                )),
                installed: false,
                ..NativeInstallOutput::default()
            }));
        }
    } else {
        // Unix: download composer.phar and make executable
        let phar_path = format!("{}/composer", install_dir.display());

        let download = exec_command!(pipe, "curl", ["-sSL", "-o", &phar_path, &phar_url,]);

        if download.exit_code != 0 {
            return Ok(Json(NativeInstallOutput {
                error: Some(format!(
                    "Failed to download composer.phar: {}",
                    download.stderr
                )),
                installed: false,
                ..NativeInstallOutput::default()
            }));
        }

        let chmod = exec_command!(pipe, "chmod", ["+x", &phar_path]);

        if chmod.exit_code != 0 {
            return Ok(Json(NativeInstallOutput {
                error: Some(format!("Failed to chmod composer: {}", chmod.stderr)),
                installed: false,
                ..NativeInstallOutput::default()
            }));
        }
    }

    Ok(Json(NativeInstallOutput {
        installed: true,
        ..NativeInstallOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    let primary: String = if env.os == HostOS::Windows {
        "composer.bat".into()
    } else {
        "composer".into()
    };

    let exes = HashMap::from_iter([("composer".into(), ExecutableConfig::new_primary(primary))]);

    let config = get_tool_config::<ComposerPluginConfig>()?;
    let mut globals_dirs = vec!["$HOME/.composer/vendor/bin".into()];
    if let Some(ref home) = config.composer_home {
        globals_dirs.insert(0, format!("{home}/vendor/bin"));
    }
    globals_dirs.push("$COMPOSER_HOME/vendor/bin".into());

    Ok(Json(LocateExecutablesOutput {
        exes,
        exes_dirs: vec![".".into()],
        globals_lookup_dirs: globals_dirs,
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn sync_shell_profile(
    Json(_): Json<SyncShellProfileInput>,
) -> FnResult<Json<SyncShellProfileOutput>> {
    let config = get_tool_config::<ComposerPluginConfig>()?;

    let export_vars = config
        .composer_home
        .as_ref()
        .map(|home| HashMap::from_iter([("COMPOSER_HOME".into(), home.clone())]));

    Ok(Json(SyncShellProfileOutput {
        check_var: "PROTO_COMPOSER_VERSION".into(),
        export_vars,
        extend_path: Some(vec!["$HOME/.composer/vendor/bin".into()]),
        skip_sync: false,
    }))
}
