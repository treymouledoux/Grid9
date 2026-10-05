# Run only on an isolated CI account: restore all touched values in finally.
$ErrorActionPreference = 'Stop'
$helper = Join-Path $PSScriptRoot 'windows-path.ps1'
$environment = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
$owner = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Software\Trey Mouledoux\Grid9')
$options = [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames
$original = $environment.GetValue('Path', $null, $options)
$originalKind = if ($null -ne $original) { $environment.GetValueKind('Path') }
$previous = $owner.GetValue('PathAddedByGrid9', $null)
$previousKind = if ($null -ne $previous) { $owner.GetValueKind('PathAddedByGrid9') }
$bin = [IO.Path]::GetFullPath((Join-Path $env:TEMP 'Grid9 path test ü'))
$other = [IO.Path]::GetFullPath((Join-Path $env:TEMP 'Grid9 moved'))
function AssertEqual($Actual, $Expected, $Message) {
    if ($Actual -cne $Expected) { throw "$Message - expected [$Expected], got [$Actual]" }
}
function ResetPath($Value) {
    $environment.SetValue('Path', $Value, [Microsoft.Win32.RegistryValueKind]::ExpandString)
    $owner.DeleteValue('PathAddedByGrid9', $false)
}
try {
    foreach ($initial in @('', 'C:\existing', '%SystemRoot%\System32;C:\other')) {
        ResetPath $initial
        & $helper -Action Add -BinDir $bin
        $expected = if ($initial) { "$initial;$bin" } else { $bin }
        AssertEqual ($environment.GetValue('Path', $null, $options)) $expected 'Add changed original entries'
        & $helper -Action Add -BinDir $bin
        AssertEqual ($environment.GetValue('Path', $null, $options)) $expected 'Repeated add duplicated entry'
        AssertEqual ($environment.GetValueKind('Path')) ([Microsoft.Win32.RegistryValueKind]::ExpandString) 'Changed registry type'
        & $helper -Action Remove -BinDir $bin
        AssertEqual ($environment.GetValue('Path', $null, $options)) $initial 'Remove changed unrelated entries'
        & $helper -Action Remove -BinDir $bin
        AssertEqual ($environment.GetValue('Path', $null, $options)) $initial 'Repeated remove changed PATH'
    }
    $existing = '"' + $bin.ToUpperInvariant() + '\"'
    ResetPath $existing
    & $helper -Action Add -BinDir $bin
    & $helper -Action Remove -BinDir $bin
    AssertEqual ($environment.GetValue('Path', $null, $options)) $existing 'Removed a pre-existing equivalent entry'
    ResetPath 'C:\keep'
    & $helper -Action Add -BinDir $bin
    & $helper -Action Add -BinDir $other
    AssertEqual ($environment.GetValue('Path', $null, $options)) "C:\keep;$other" 'Moving installation left old entry'
    & $helper -Action Remove -BinDir $bin
    AssertEqual ($environment.GetValue('Path', $null, $options)) "C:\keep;$other" 'Old uninstaller removed current entry'
    & $helper -Action Remove -BinDir $other
    AssertEqual ($environment.GetValue('Path', $null, $options)) 'C:\keep' 'Final cleanup failed'
    Write-Host 'PATH cases passed: empty, single, multiple, expandable, repeated, pre-existing, and moved.'
} finally {
    if ($null -eq $original) { $environment.DeleteValue('Path', $false) }
    else { $environment.SetValue('Path', $original, $originalKind) }
    if ($null -eq $previous) { $owner.DeleteValue('PathAddedByGrid9', $false) }
    else { $owner.SetValue('PathAddedByGrid9', $previous, $previousKind) }
    $environment.Dispose()
    $owner.Dispose()
}
