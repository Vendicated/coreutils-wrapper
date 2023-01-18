$coreutils_cmds_list = @(%CMD_LIST_PLACEHOLDER%)

foreach ($cmd in $coreutils_cmds_list) {
    if (Test-Path -LiteralPath Alias:$cmd) {
        Remove-Alias -Force $cmd
    }
}

Remove-Variable coreutils_cmds_list
