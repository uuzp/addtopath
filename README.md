# AddToPath - Windows PATH 管理工具

一个简单的 Windows 命令行工具，用于将文件夹路径添加到当前用户的 PATH 环境变量，并提供方便的右键菜单集成。

## 功能

*   **命令行添加路径:** 将一个或多个指定的文件夹路径添加到用户 PATH。
*   **双击添加自身路径:** 直接双击 `addtopath.exe` 可将其所在的目录添加到用户 PATH。
*   **右键菜单集成:**
    *   通过 `--reg` 参数注册右键菜单项（需要管理员权限）。
    *   注册后，右键单击文件夹或文件夹空白处，选择“添加到用户 PATH”即可将该文件夹添加到用户 PATH。
    *   通过 `--unreg` 参数移除右键菜单项（需要管理员权限）。
*   **自动提权:** 在执行 `--reg` 或 `--unreg` 操作时，会自动请求管理员权限（弹出 UAC 确认框）。
*   **静默运行:** 程序在后台运行，不会显示控制台窗口。

## 使用方法

### 编译

如果你有 Rust 环境，可以在项目根目录运行以下命令编译优化后的 release 版本：

```bash
cargo build --release
```

编译后的可执行文件位于 `target\release\addtopath.exe`。

### 添加指定路径

在命令提示符或 PowerShell 中运行：

```cmd
# 添加单个路径
target\release\addtopath.exe "C:\Program Files\MyTool\bin"

# 添加多个路径
target\release\addtopath.exe "C:\path\to\tool1" "D:\another path\with spaces"
```

程序会将不存在于当前用户 PATH 中的路径追加到末尾。

### 添加程序自身目录

直接在文件资源管理器中双击 `target\release\addtopath.exe` 文件。程序会自动获取其所在的目录 (`target\release`) 并尝试将其添加到用户 PATH。

### 注册右键菜单

**必须以管理员身份运行！**

1.  打开一个**管理员权限**的命令提示符或 PowerShell。
2.  导航到 `addtopath.exe` 所在的目录（例如 `cd a:\Dev\PJ\addtopath\target\release`）。
3.  运行：
    ```cmd
    addtopath.exe --reg
    ```
4.  程序会注册右键菜单项。

### 使用右键菜单

注册成功后：
*   右键单击任意**文件夹**，选择“添加到用户 PATH”。
*   在文件夹内的**空白区域**右键单击，选择“添加到用户 PATH”。

该文件夹的路径将被添加到用户 PATH。

### 注销右键菜单

**必须以管理员身份运行！**

1.  打开一个**管理员权限**的命令提示符或 PowerShell。
2.  导航到 `addtopath.exe` 所在的目录。
3.  运行：
    ```cmd
    addtopath.exe --unreg
    ```
4.  程序会移除之前添加的右键菜单项。

## 注意事项

*   **管理员权限:** 注册 (`--reg`) 和注销 (`--unreg`) 右键菜单需要管理员权限。程序会自动触发 UAC 提示。
*   **PATH 生效:** 对 PATH 环境变量的更改可能需要重新启动命令提示符、PowerShell、应用程序，甚至注销/登录或重启系统才能完全生效。
*   **文件位置:** 注册右键菜单后，请不要移动或删除 `addtopath.exe` 文件，否则右键菜单项会失效。如果需要移动，请先运行 `--unreg` 注销，移动后再重新运行 `--reg` 注册。
*   **错误信息:** 由于程序以 `windows_subsystem` 模式运行（无控制台窗口），错误信息主要通过 `stderr` 输出，可能不容易看到。关键操作失败时程序会以非零代码退出。