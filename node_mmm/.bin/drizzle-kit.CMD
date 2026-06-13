@SETLOCAL
@IF NOT DEFINED NODE_PATH (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\drizzle-kit@0.31.8\node_modules\drizzle-kit\node_modules;D:\worker\qfd\node_modules\.pnpm\drizzle-kit@0.31.8\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules"
) ELSE (
  @SET "NODE_PATH=D:\worker\qfd\node_modules\.pnpm\drizzle-kit@0.31.8\node_modules\drizzle-kit\node_modules;D:\worker\qfd\node_modules\.pnpm\drizzle-kit@0.31.8\node_modules;D:\worker\qfd\node_modules\.pnpm\node_modules;%NODE_PATH%"
)
@IF EXIST "%~dp0\node.exe" (
  "%~dp0\node.exe"  "%~dp0\..\drizzle-kit\bin.cjs" %*
) ELSE (
  @SET PATHEXT=%PATHEXT:;.JS;=;%
  node  "%~dp0\..\drizzle-kit\bin.cjs" %*
)
