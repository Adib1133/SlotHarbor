[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $Executable,

    [ValidateSet('x64', 'x86')]
    [string] $Architecture = 'x64'
)

$ErrorActionPreference = 'Stop'
$binary = (Resolve-Path -LiteralPath $Executable).Path
$project = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$target = @{
    x64 = 'x86_64-pc-windows-msvc'
    x86 = 'i686-pc-windows-msvc'
}[$Architecture]
$machine = @{ x64 = 0x8664; x86 = 0x014c }[$Architecture]

$reader = [IO.BinaryReader]::new([IO.File]::OpenRead($binary))
try {
    $reader.BaseStream.Position = 0x3c
    $reader.BaseStream.Position = $reader.ReadInt32()
    if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne $machine) {
        throw "The executable is not a Windows $Architecture binary."
    }
} finally {
    $reader.Dispose()
}

Push-Location -LiteralPath $project
try {
    $metadataJson = cargo metadata --locked --format-version 1 --filter-platform $target
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the locked Cargo dependency metadata.' }
    $metadata = ($metadataJson -join "`n") | ConvertFrom-Json
} finally {
    Pop-Location
}
$root = $metadata.packages | Where-Object id -EQ $metadata.resolve.root
$version = $root.version
if (-not $version -or $root.name -ne 'slotharbor') { throw 'Could not determine the SlotHarbor package version.' }

$output = Join-Path $project 'dist'
New-Item -ItemType Directory -Path $output -Force | Out-Null
$output = (Resolve-Path -LiteralPath $output).Path
$stage = Join-Path $output ('.package-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage | Out-Null

try {
    Copy-Item -LiteralPath $binary -Destination (Join-Path $stage 'slotharbor.exe')
    foreach ($name in @('LICENSE', 'THIRD_PARTY_NOTICES.md', 'Cargo.lock', 'licenses')) {
        Copy-Item -LiteralPath (Join-Path $project $name) -Destination $stage -Recurse
    }
    $dependencyLicenses = Join-Path $stage 'licenses\rust'
    New-Item -ItemType Directory -Path $dependencyLicenses | Out-Null
    $records = foreach ($crate in ($metadata.packages | Where-Object id -NE $root.id | Sort-Object name, version)) {
        $crateRoot = Split-Path -LiteralPath $crate.manifest_path
        $noticeFiles = @(Get-ChildItem -LiteralPath $crateRoot -File |
            Where-Object Name -Match '^(license|licence|copying|notice|copyright)($|[-_.])')
        if ($crate.license_file) {
            $declared = Join-Path $crateRoot $crate.license_file
            if (Test-Path -LiteralPath $declared -PathType Leaf) {
                $noticeFiles += Get-Item -LiteralPath $declared
            }
        }
        $noticeFolder = Join-Path $crateRoot 'LICENSES'
        if ($noticeFiles.Count -gt 0 -or (Test-Path -LiteralPath $noticeFolder -PathType Container)) {
            $crateDestination = Join-Path $dependencyLicenses "$($crate.name)-$($crate.version)"
            New-Item -ItemType Directory -Path $crateDestination -Force | Out-Null
            foreach ($notice in ($noticeFiles | Sort-Object FullName -Unique)) {
                Copy-Item -LiteralPath $notice.FullName -Destination $crateDestination
            }
            if (Test-Path -LiteralPath $noticeFolder -PathType Container) {
                Copy-Item -LiteralPath $noticeFolder -Destination $crateDestination -Recurse
            }
        }
        [pscustomobject]@{
            Package = $crate.name
            Version = $crate.version
            License = $crate.license
            Repository = $crate.repository
        }
    }
    $records | Export-Csv -NoTypeInformation -Encoding UTF8 -Path (Join-Path $dependencyLicenses 'dependencies.csv')

    $archiveName = "SlotHarbor-v$version-windows-$Architecture.zip"
    $archive = Join-Path $output $archiveName
    $zipEpoch = [datetime]::new(1980, 1, 1)
    Get-ChildItem -LiteralPath $stage -Recurse -File | Where-Object LastWriteTime -LT $zipEpoch |
        ForEach-Object { $_.LastWriteTime = $zipEpoch }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $archive -Force
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -LiteralPath "$archive.sha256" -Value "$hash  $archiveName" -Encoding ascii
    Copy-Item -LiteralPath $binary -Destination (Join-Path $output 'slotharbor.exe') -Force
    Write-Output $archive
} finally {
    $resolvedStage = (Resolve-Path -LiteralPath $stage).Path
    if ((Split-Path -LiteralPath $resolvedStage) -eq $output -and $resolvedStage -eq $stage) {
        Remove-Item -LiteralPath $resolvedStage -Recurse -Force
    }
}
