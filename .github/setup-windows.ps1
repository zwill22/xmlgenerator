Write-Output "Windows action setup"
Write-Output "--------------------"

Write-Output "Installing libxml2 with vcpkg..."
vcpkg install libxml2
Write-Output "✔ libxml2 installed successfully."

Write-Output "Integrating..."
vcpkg integrate install
Write-Output "✔ Integration successful"

$arch = $Env:RUNNER_ARCH.ToLower()
$root = $Env:VCPKG_INSTALLATION_ROOT
Write-Output ""
Write-Output "Runner architecture: $arch"
Write-Output "vcpkg installation root: $root"


$toolchain = "$root\scripts\buildsystems\vcpkg.cmake"
Write-Output "CMAKE_TOOLCHAIN_FILE = $toolchain"
Write-Output "CMAKE_TOOLCHAIN_FILE=$toolchain" >> $env:GITHUB_ENV

$include = "$root\installed\$arch-windows\include\libxml2\"
Write-Output "LIBXML2_INCLUDE_DIR = $include"
Write-Output "LIBXML2_INCLUDE_DIR=$include" >> $env:GITHUB_ENV

$lib_dir = "$root\installed\$arch-windows\lib\"
Write-Output "LIBXML2_LIBRARY_DIR = $lib_dir"
Write-Output "LIBXML2_LIBRARY_DIR=$lib_dir" >> $env:GITHUB_ENV

$dll_dir = "$root\installed\$arch-windows\lib\"
Write-Output "LIBXML2_DLL_DIR = $dll_dir"
$dll_dir | Out-File -FilePath $env:GITHUB_PATH -Append
Write-Output "LIBXML2_DLL_DIR added to PATH"

Write-Output ""
Write-Output "Setup complete ✔"
