set accountName to short user name of (system info)
display dialog "Remove Grid9 and its terminal commands? This also deletes your Grid9 documentation, examples, logs, cache, and any personal files in its Application Support folder." buttons {"Cancel", "Uninstall"} default button "Cancel" with icon caution
if button returned of result is "Uninstall" then
    do shell script "/usr/local/bin/grid9-uninstall --user " & quoted form of accountName with administrator privileges
    display dialog "Grid9 has been removed." buttons {"OK"} default button "OK"
end if
