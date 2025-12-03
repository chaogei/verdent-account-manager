fn main() {
    // Windows特定配置：请求管理员权限
    // 注意：这将使得应用程序启动时总是请求管理员权限
    // 如果不需要管理员权限，可以注释掉下面的代码
    #[cfg(windows)]
    {
        use std::env;

        // 通过环境变量控制是否需要管理员权限
        let require_admin = env::var("REQUIRE_ADMIN").unwrap_or_else(|_| "true".to_string());

        if require_admin == "true" {
            let mut windows = tauri_build::WindowsAttributes::new();
            // 嵌入完整的管理员权限清单
            // 必须包含 dependency 和 compatibility 部分，否则会导致 TaskDialogIndirect 等 API 调用失败
            windows = windows.app_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.5.2.0"
    processorArchitecture="*"
    name="ai.verdent.account-manager"
    type="win32"
  />
  <description>Verdent Account Manager</description>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows Vista -->
      <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}"/>
      <!-- Windows 7 -->
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
      <!-- Windows 8 -->
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <!-- Windows 8.1 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <!-- Windows 10 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>
            "#);

            tauri_build::try_build(
                tauri_build::Attributes::new()
                    .windows_attributes(windows)
            ).expect("failed to run build script");
        } else {
            tauri_build::build();
        }
    }

    #[cfg(not(windows))]
    {
        tauri_build::build();
    }
}
