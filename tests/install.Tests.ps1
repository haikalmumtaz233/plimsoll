BeforeAll {
    $scriptPath = Join-Path $PSScriptRoot '..' 'scripts' 'install.ps1'
    $parseErrors = $null
    $scriptAst = [System.Management.Automation.Language.Parser]::ParseFile($scriptPath, [ref] $null, [ref] $parseErrors)
    $isFunction = { param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] }
    foreach ($definition in $scriptAst.FindAll($isFunction, $true)) {
        . ([scriptblock]::Create($definition.Extent.Text))
    }
    $x64Hash = 'a' * 64
    $arm64Hash = 'B' * 64
    $checksums = @(
        "$x64Hash  Plimsoll_1.0.0_x64-setup.exe",
        "$arm64Hash *Plimsoll_1.0.0_arm64-setup.exe",
        "$('c' * 64)  Plimsoll_1.0.0_x64_en-US.msi",
        "$('d' * 64)  install.ps1"
    ) -join "`r`n"
}

Describe 'install.ps1' {
    It 'parses without errors' {
        $parseErrors | Should -BeNullOrEmpty
    }

    It 'never exits the host session' {
        $isExit = { param($node) $node -is [System.Management.Automation.Language.ExitStatementAst] }
        $scriptAst.FindAll($isExit, $true) | Should -BeNullOrEmpty
    }

    It 'contains only ASCII characters' {
        $text = [System.IO.File]::ReadAllText($scriptPath)
        $text | Should -Not -Match '[^\x00-\x7F]'
    }
}

Describe 'Get-PlimsollReleaseBase' {
    It 'uses the latest release when no version is given' {
        Get-PlimsollReleaseBase -Repository 'owner/app' -Version '' |
            Should -Be 'https://github.com/owner/app/releases/latest/download'
    }

    It 'uses the tagged release when a version is given' {
        Get-PlimsollReleaseBase -Repository 'owner/app' -Version 'v1.2.3-rc.1' |
            Should -Be 'https://github.com/owner/app/releases/download/v1.2.3-rc.1'
    }

    It 'rejects a version that is not a release tag' {
        { Get-PlimsollReleaseBase -Repository 'owner/app' -Version '../evil' } | Should -Throw '*release tag*'
    }
}

Describe 'Get-PlimsollArchitecture' {
    It 'maps x64 processors to x64' {
        Get-PlimsollArchitecture -ProcessorArchitecture 9 | Should -Be 'x64'
    }

    It 'maps ARM64 processors to arm64' {
        Get-PlimsollArchitecture -ProcessorArchitecture 12 | Should -Be 'arm64'
    }

    It 'rejects 32-bit processors' {
        { Get-PlimsollArchitecture -ProcessorArchitecture 0 } | Should -Throw '*64-bit*'
    }
}

Describe 'Select-PlimsollInstaller' {
    It 'picks the x64 setup and its hash' {
        $installer = Select-PlimsollInstaller -Checksums $checksums -Architecture 'x64'
        $installer.Name | Should -Be 'Plimsoll_1.0.0_x64-setup.exe'
        $installer.Hash | Should -Be $x64Hash
    }

    It 'picks the arm64 setup and lowercases its hash' {
        $installer = Select-PlimsollInstaller -Checksums $checksums -Architecture 'arm64'
        $installer.Name | Should -Be 'Plimsoll_1.0.0_arm64-setup.exe'
        $installer.Hash | Should -Be $arm64Hash.ToLowerInvariant()
    }

    It 'fails when the release has no setup for the architecture' {
        $x64Only = "$x64Hash  Plimsoll_1.0.0_x64-setup.exe"
        { Select-PlimsollInstaller -Checksums $x64Only -Architecture 'arm64' } | Should -Throw '*arm64*'
    }

    It 'fails when the release lists more than one setup for the architecture' {
        $duplicated = "$checksums`n$('e' * 64)  Plimsoll_1.0.1_x64-setup.exe"
        { Select-PlimsollInstaller -Checksums $duplicated -Architecture 'x64' } | Should -Throw '*x64*'
    }

    It 'ignores names that could escape the download folder' {
        $escaping = "$x64Hash  ..\Plimsoll_1.0.0_x64-setup.exe"
        { Select-PlimsollInstaller -Checksums $escaping -Architecture 'x64' } | Should -Throw
    }
}

Describe 'Assert-PlimsollChecksum' {
    BeforeAll {
        $payloadPath = Join-Path $TestDrive 'payload.bin'
        [System.IO.File]::WriteAllText($payloadPath, 'plimsoll')
        $payloadHash = (Get-FileHash -LiteralPath $payloadPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    It 'accepts a matching hash' {
        { Assert-PlimsollChecksum -Path $payloadPath -Expected $payloadHash } | Should -Not -Throw
    }

    It 'rejects a different hash' {
        { Assert-PlimsollChecksum -Path $payloadPath -Expected ('0' * 64) } | Should -Throw '*checksum*'
    }
}
