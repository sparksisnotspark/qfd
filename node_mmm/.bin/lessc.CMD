@SETLOCAL
@IF NOT DEFINED NODE_PATH (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules\less\bin\node_modules;D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules\less\node_modules;D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules"
) ELSE (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules\less\bin\node_modules;D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules\less\node_modules;D:\worker\qfd\node_modules\.pnpm\less@4.5.1\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules;%NODE_PATH%"
)
@IF EXIST "%~dp0\node.exe" (
  "%~dp0\node.exe"  "%~dp0\..\less\bin\lessc" %*
) ELSE (
  @SET PATHEXT=%PATHEXT:;.JS;=;%
  node  "%~dp0\..\less\bin\lessc" %*
)
