set accountName to short user name of (system info)

activate

tell application "System Events" to activate
set theChoice to button returned of (display dialog "Remove Grid9 and its components? This also removes your entire Grid9 folder in Application Support, including documentation, examples, personal files, logs, and cache files." buttons {"Cancel", "Uninstall"} default button "Cancel" with icon caution)

if theChoice is "Uninstall" then
	try
		do shell script "/usr/local/bin/grid9-uninstall --user " & quoted form of accountName with administrator privileges
		tell application "System Events" to activate
		display dialog "Grid9 has been removed." buttons {"OK"} default button "OK"
	on error errMsg number errNum
		if errNum is not -128 then -- -128 = user cancelled the auth prompt
			tell application "System Events" to activate
			display dialog "Uninstall failed:" & return & errMsg buttons {"OK"} default button "OK" with icon stop
		end if
	end try
end if
