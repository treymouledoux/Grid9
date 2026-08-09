$url = "https://github.com/dom96/choosenim/releases/download/v0.8.4/choosenim-0.8.4_windows_amd64.exe"

Write-Output "Installing Nim... (0/5)"
Invoke-WebRequest -Uri $url -OutFile "choosenim.exe"
.\choosenim stable --firstInstall

Write-Output "Installing dependencies... (1/5)"
nimble install docopt -y
nimble install parsetoml -y

Write-Output "Compiling Grid9 and Grid9Converter... (2/5)"
git clone https://github.com/treymouledoux/Grid9.git
Set-Location Grid9/src
nim c -d:release main.nim
nim c -d:release Grid9Converter.nim

Write-Output "Installing Grid9 and Grid9Converter... (3/5)"
Rename-Item -Path "main.exe" -NewName "Grid9.exe"
New-Item "C:\Program Files (x86)\Grid9" -itemType Directory
Move-Item -Path "Grid9.exe" -Destination "C:\Program Files (x86)\Grid9"
Move-Item -Path "Grid9Converter.exe" -Destination "C:\Program Files (x86)\Grid9"
New-Item "C:\ProgramData\Grid9" -itemType Directory

Write-Output "Installing Examples... (4/5)"
Move-Item -Path "components\examples" -Destination "C:\ProgramData\Grid9"

Write-Output "Installing documentation... (5/5)"
Move-Item -Path "components\documentation" -Destination "C:\ProgramData\Grid9"

Write-Output "Finished!"