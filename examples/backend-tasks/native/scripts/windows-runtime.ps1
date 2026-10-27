# Read the installed build tool's official redistributable directory. Never copy
# arbitrary DLLs from PATH/System32 or change the machine's runtime installation.
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Security/Microsoft.PowerShell.Security.psd1') -ErrorAction Stop
Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Utility/Microsoft.PowerShell.Utility.psd1') -ErrorAction Stop
$runtimeDestination = (Resolve-Path -LiteralPath $env:TSRUST_DESKTOP_RUNTIME_DEST).Path
$runtimeArchitecture = $env:TSRUST_DESKTOP_RUNTIME_ARCH
if ($runtimeArchitecture -notin @('x64', 'arm64')) { throw 'Unsupported desktop runtime architecture' }
$vsLocator = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (-not (Test-Path -LiteralPath $vsLocator)) { throw 'Visual Studio redistributable discovery requires vswhere.exe' }
$vsDirectory = & $vsLocator -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vsDirectory) { throw 'Install the Microsoft C++ build tools and redistributable build files' }
$runtimeCandidates = Get-ChildItem -Path (Join-Path $vsDirectory "VC/Redist/MSVC/*/$runtimeArchitecture/Microsoft.VC*.CRT") -Directory | ForEach-Object {
    $runtimeFile = Join-Path $_.FullName 'vcruntime140.dll'
    if (Test-Path -LiteralPath $runtimeFile) {
        [pscustomobject]@{ Directory = $_.FullName; Version = [version](Get-Item -LiteralPath $runtimeFile).VersionInfo.FileVersion }
    }
}
$runtimeSource = $runtimeCandidates | Sort-Object Version -Descending | Select-Object -First 1
if (-not $runtimeSource) { throw 'No supported app-local Microsoft C++ runtime was found' }
$runtimeFiles = @(Get-ChildItem -LiteralPath $runtimeSource.Directory -Filter '*.dll' -File | Sort-Object Name)
$runtimeRecords = foreach ($runtimeFile in $runtimeFiles) {
    $signature = Get-AuthenticodeSignature -LiteralPath $runtimeFile.FullName
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)') {
        throw "Unverified Microsoft redistributable: $($runtimeFile.Name)"
    }
    Copy-Item -LiteralPath $runtimeFile.FullName -Destination (Join-Path $runtimeDestination $runtimeFile.Name) -Force
    [pscustomobject]@{ name = $runtimeFile.Name; version = $runtimeFile.VersionInfo.FileVersion;
        sha256 = (Get-FileHash -LiteralPath $runtimeFile.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); publisher = 'Microsoft Corporation' }
}
[pscustomobject]@{ schemaVersion = 1; component = 'Microsoft Visual C++ app-local runtime'; architecture = $runtimeArchitecture; files = @($runtimeRecords) } |
    ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runtimeDestination 'runtime-provenance.json') -Encoding UTF8
@'
Microsoft Visual C++ Runtime

These unmodified Microsoft-signed binaries are copied from the installed Visual
Studio redistributable directory for application-local deployment. Redistribution
is governed by the applicable Microsoft Visual Studio software license terms and
distributable-code list; the application's MIT license does not cover these files.

https://learn.microsoft.com/cpp/windows/redistributing-visual-cpp-files
https://learn.microsoft.com/visualstudio/releases/2022/redistribution

App-local runtimes are updated with the application. The publisher must rebuild
and distribute relevant Microsoft runtime security updates with app updates.
'@ | Set-Content -LiteralPath (Join-Path $runtimeDestination 'NOTICE-Microsoft-runtime.txt') -Encoding UTF8
