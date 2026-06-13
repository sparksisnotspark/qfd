@SETLOCAL
@IF NOT DEFINED NODE_PATH (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules\vite\bin\node_modules;D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules\vite\node_modules;D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules"
) ELSE (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules\vite\bin\node_modules;D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules\vite\node_modules;D:\worker\qfd\node_modules\.pnpm\vite@6.4.1_less@4.5.1\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules;%NODE_PATH%"
)
@IF EXIST "%~dp0\node.exe" (
  "%~dp0\node.exe"  "%~dp0\..\vite\bin\vite.js" %*
) ELSE (
  @SET PATHEXT=%PATHEXT:;.JS;=;%
  node  "%~dp0\..\vite\bin\vite.js" %*
)
