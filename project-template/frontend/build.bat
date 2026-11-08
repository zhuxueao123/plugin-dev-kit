@echo off
setlocal

if not exist node_modules ( 
  echo Installing dependencies...
  npm install
)

echo Building frontend plugin bundles...
npm run build

if errorlevel 1 (
  echo Build failed.
  exit /b 1
)

echo Build completed.
