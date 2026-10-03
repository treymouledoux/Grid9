param(
    [Parameter(Mandatory = $true)][ValidateSet('Add', 'Remove')][string]$Action,
    [Parameter(Mandatory = $true)][string]$BinDir
)

$ErrorActionPreference = 'Stop'
$directory = [IO.Path]::GetFullPath($BinDir).TrimEnd('\')
$registryKey = 'HKCU:\Software\Trey Mouledoux\Grid9'
$ownershipValue = 'PathAddedByGrid9'
$previous = Get-ItemPropertyValue -Path $registryKey -Name $ownershipValue -ErrorAction SilentlyContinue
$original = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = if ([string]::IsNullOrEmpty($original)) { @() } else { @($original -split ';') }

function MatchesDirectory([string]$Entry, [string]$Directory) {
    return [string]::Equals(
        [Environment]::ExpandEnvironmentVariables($Entry.Trim().Trim('"')).TrimEnd('\'),
        $Directory, [StringComparison]::OrdinalIgnoreCase
    )
}

if ($Action -eq 'Add') {
    # If a previous installation moved, remove only the entry we added.
    if ($previous -and -not (MatchesDirectory $previous $directory)) {
        $entries = @($entries | Where-Object { -not (MatchesDirectory $_ $previous) })
        Remove-ItemProperty -Path $registryKey -Name $ownershipValue -ErrorAction SilentlyContinue
    }
    $present = @($entries | Where-Object { MatchesDirectory $_ $directory }).Count -gt 0
    if (-not $present) {
        $entries += $directory
        New-Item -Path $registryKey -Force | Out-Null
        New-ItemProperty -Path $registryKey -Name $ownershipValue -Value $directory -PropertyType String -Force | Out-Null
    }
} elseif ($previous -and (MatchesDirectory $previous $directory)) {
    # Preserve pre-existing user PATH entries that the installer did not add.
    $entries = @($entries | Where-Object { -not (MatchesDirectory $_ $directory) })
    Remove-ItemProperty -Path $registryKey -Name $ownershipValue
}

$updated = $entries -join ';'
if ($updated -ne $original) {
    [Environment]::SetEnvironmentVariable('Path', $updated, 'User')
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Grid9Environment {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr window, uint message, UIntPtr wParam, string lParam,
        uint flags, uint timeout, out UIntPtr result);
}
'@
    $result = [UIntPtr]::Zero
    [Grid9Environment]::SendMessageTimeout(
        [IntPtr]0xffff, 0x1a, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result
    ) | Out-Null
}
