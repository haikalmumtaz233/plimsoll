& {
    Set-StrictMode -Version 3.0
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'
    $InformationPreference = 'Continue'

    function Get-PlimsollReleaseBase {
        param(
            [Parameter(Mandatory)] [string] $Repository,
            [AllowEmptyString()] [string] $Version
        )
        if ([string]::IsNullOrWhiteSpace($Version)) {
            return "https://github.com/$Repository/releases/latest/download"
        }
        if ($Version -cnotmatch '^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$') {
            throw "PLIMSOLL_VERSION must be a release tag such as v1.0.0, not '$Version'."
        }
        return "https://github.com/$Repository/releases/download/$Version"
    }

    function Get-PlimsollArchitecture {
        param([Parameter(Mandatory)] [int] $ProcessorArchitecture)
        switch ($ProcessorArchitecture) {
            9 { return 'x64' }
            12 { return 'arm64' }
            default { throw 'Plimsoll needs 64-bit Windows on an x64 or ARM64 processor.' }
        }
    }

    function Select-PlimsollInstaller {
        param(
            [Parameter(Mandatory)] [string] $Checksums,
            [Parameter(Mandatory)] [ValidateSet('x64', 'arm64')] [string] $Architecture
        )
        $pattern = "^(?<hash>[0-9A-Fa-f]{64}) [ *](?<name>Plimsoll_[0-9A-Za-z.-]+_$Architecture-setup\.exe)$"
        $found = @(
            foreach ($line in $Checksums -split '\r?\n') {
                $match = [regex]::Match($line.Trim(), $pattern)
                if ($match.Success) {
                    [pscustomobject]@{
                        Name = $match.Groups['name'].Value
                        Hash = $match.Groups['hash'].Value.ToLowerInvariant()
                    }
                }
            }
        )
        if ($found.Count -ne 1) {
            throw "Expected one $Architecture installer in the release checksums, found $($found.Count)."
        }
        return $found[0]
    }

    function Assert-PlimsollChecksum {
        param(
            [Parameter(Mandatory)] [string] $Path,
            [Parameter(Mandatory)] [string] $Expected
        )
        $sha256 = [System.Security.Cryptography.SHA256]::Create()
        $stream = [System.IO.File]::OpenRead($Path)
        try {
            $digest = $sha256.ComputeHash($stream)
        }
        finally {
            $stream.Dispose()
            $sha256.Dispose()
        }
        $actual = -join ($digest | ForEach-Object { $_.ToString('x2') })
        if ($actual -ne $Expected) {
            throw 'The downloaded installer does not match the release checksum. Nothing was installed. Please try again.'
        }
    }

    function Close-PlimsollApp {
        param([Parameter(Mandatory)] [string] $InstallRoot)
        $running = Get-Process -Name 'plimsoll' -ErrorAction SilentlyContinue | Where-Object {
            $_.Path -and $_.Path.StartsWith($InstallRoot, [System.StringComparison]::OrdinalIgnoreCase)
        }
        if ($running) {
            Write-Information 'Closing the running Plimsoll...'
            $running | Stop-Process -Force -Confirm:$false
            $running | Wait-Process -Timeout 10 -ErrorAction SilentlyContinue
        }
    }

    function Install-Plimsoll {
        $base = Get-PlimsollReleaseBase -Repository 'haikalmumtaz233/plimsoll' -Version $env:PLIMSOLL_VERSION
        $processor = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
        $architecture = Get-PlimsollArchitecture -ProcessorArchitecture $processor.Architecture
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

        $workspace = Join-Path ([System.IO.Path]::GetTempPath()) ('plimsoll-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $workspace | Out-Null
        try {
            Write-Information 'Finding the Plimsoll release...'
            $checksumsPath = Join-Path $workspace 'SHA256SUMS'
            Invoke-WebRequest -Uri "$base/SHA256SUMS" -OutFile $checksumsPath -UseBasicParsing
            $checksums = [System.IO.File]::ReadAllText($checksumsPath)
            $installer = Select-PlimsollInstaller -Checksums $checksums -Architecture $architecture

            Write-Information "Downloading $($installer.Name)..."
            $installerPath = Join-Path $workspace $installer.Name
            Invoke-WebRequest -Uri "$base/$($installer.Name)" -OutFile $installerPath -UseBasicParsing
            Assert-PlimsollChecksum -Path $installerPath -Expected $installer.Hash

            Close-PlimsollApp -InstallRoot (Join-Path $env:LOCALAPPDATA 'Plimsoll\')
            Write-Information 'Installing for the current user...'
            $setup = Start-Process -FilePath $installerPath -ArgumentList '/S', '/R' -PassThru
            $null = $setup.Handle
            $setup.WaitForExit()
            if ($setup.ExitCode -ne 0) {
                throw "The installer stopped with exit code $($setup.ExitCode)."
            }
            Write-Information 'Plimsoll is installed. Look for its icon in the system tray.'
        }
        finally {
            Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Install-Plimsoll
}
