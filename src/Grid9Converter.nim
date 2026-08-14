from toml_manager import nil
from glyphs import nil

import strformat, docopt, times, os, re
import std/strutils
import std/terminal

when defined windows:
    const
        subMainDir = r"C:\ProgramData\Grid9\"
        mainDir = r"C:\ProgramData\Grid9\grid9converter\"
        logDir = r"C:\ProgramData\Grid9\grid9converter\logs\"
else:
    const
        subMainDir = "/usr/share/Grid9/"
        mainDir = "/usr/share/Grid9/grid9converter/"
        logDir = "/usr/share/Grid9/grid9converter/logs/"

const doc = """
Usage:
    Grid9Converter (about | a)
    Grid9Converter (version | v)
    Grid9Converter (convert | c) <path> <conversion>
"""

const converterversion = "2026-001"

proc logThis(mode: string, message: string) : string {.discardable.} =
    let
        time = now().format("yyyy-MM-dd HH:mm:ss")
        logFile = open(logDir & now().format("yyyy-MM-dd") & ".log", fmAppend)
    case mode
    of "INFO":
        stdout.styledWriteLine(fgCyan, mode, fgWhite, " ", message)
    of "WARNING":
        stdout.styledWriteLine(fgYellow, mode, fgWhite, " ", message)
    of "ERROR":
        stdout.styledWriteLine(fgRed, mode, fgWhite, " ", message)
    logFile.writeLine(fmt"{time} - {mode} - {message}")

proc about() =
    echo "\nGrid9Converter is tool for the Grid9 programming language that is used to convert your Grid9 projects to other formats.\n"

proc version() =
    echo "\n", converterversion, "\n"

proc clean() =
    os.removeDir(logDir)
    os.createDir(logDir)
    logThis("INFO", "Cleaned log dir")

proc convert(path: string, conversion: string) =
    var path = path
    if os.fileExists(path):
        discard
    elif os.fileExists(path & ".g9"):
        path = path & ".g9"
    else:
        let errorMessage = "File not found at '" & path & "' maybe check your path.\n"
        logThis("ERROR", errorMessage)
        discard errorMessage
        return

    case conversion
    of "textToGrid9":
        logThis("INFO", "Converting text to Grid9")
        logThis("INFO", "Reading config file")

        if os.fileExists(replace(path, re".g9", ".toml")):
            let
                tomlPath = replace(path, re".g9", ".toml")
                config = toml_manager.getConfig(tomlPath)
            var minGrid9Ver = config[7]
            if minGrid9Ver > "2022-013":
                logThis("WARNING", "This conversion method is not supported for Grid9 versions below 2022-013, the output may not work as expected, continue? (y/n)")
                let answer = readLine(stdin)
                if answer != "y":
                    discard answer
                    return
                discard answer

        let inputFile = open(path, fmRead)
        var inputData = inputFile.readAll()
        inputFile.close()
        var outputData = ""

        for currentChar in inputData:
            #find current character in glyph table
            var grid = "000000000"
            var charFound = false
            var currentCharNum = 0
            while currentCharNum < 255:
                let currentCharBin = toBin(currentCharNum, 9)
                if $currentChar == $glyphs.get_glyph($currentCharBin):
                    logThis("INFO", "FoundChar " & $glyphs.get_glyph($currentCharBin))
                    charFound = true
                    
                    #convert the bin to grid9
                    var cellNum = 0
                    while cellNum < 9:
                        var letter = currentCharBin[cellNum]

                        #if $letter == "0":
                        #    outputData &= "s" & $cellNum & "0"
                        #elif $letter == "1":
                        #    outputData &= "s" & $cellNum & "1"

                        if grid[cellNum] == letter:
                            discard
                        else:
                            outputData &= "f" & $cellNum

                        discard letter
                        cellNum += 1

                    outputData &= "qsa0"

                    break
                currentCharNum += 1
            if charFound == false:
                let errorMessage = "The character '" & currentChar & "' was not found in the glyphs table.\n"
                logThis("ERROR", errorMessage)
                discard errorMessage
        
        logThis("INFO", "Conversion completed...\n\n" & outputData & "p\n")
        let exit = readLine(stdin)
        discard exit

    of "grid9ToRetroGadget":
        logThis("INFO", "Converting Grid9 to RetroGadget Grid9")
        logThis("INFO", "Reading config file")
        
        if os.fileExists(replace(path, re".g9", ".toml")):
            let
                tomlPath = replace(path, re".g9", ".toml")
                config = toml_manager.getConfig(tomlPath)
            var minGrid9Ver = config[7]
            if minGrid9Ver > "2022.013":
                logThis("WARNING", "This conversion method is not supported for Grid9 versions below 2022.020, the output may not work as expected, continue? (y/n)")
                let answer = readLine(stdin)
                if answer != "y":
                    discard answer
                    return
                discard answer

        let inputFile = open(path, fmRead)
        var inputData = inputFile.readAll()
        inputFile.close()

        inputData = replace(inputData, re("[A-Z*()\t ]+"), "")
        inputData = replace(inputData, "b0", "")
        inputData = replace(inputData, "f0", "f1")
        inputData = replace(inputData, "f1", "f2")
        inputData = replace(inputData, "f2", "f3")
        inputData = replace(inputData, "f3", "f4")
        inputData = replace(inputData, "f4", "f5")
        inputData = replace(inputData, "f5", "f6")
        inputData = replace(inputData, "f6", "f7")
        inputData = replace(inputData, "f7", "f8")
        inputData = replace(inputData, "f8", "f9")
        inputData = replace(inputData, "i0", "i1")
        inputData = replace(inputData, "i1", "i2")
        inputData = replace(inputData, "i2", "i3")
        inputData = replace(inputData, "i3", "i4")
        inputData = replace(inputData, "i4", "i5")
        inputData = replace(inputData, "i5", "i6")
        inputData = replace(inputData, "i6", "i7")
        inputData = replace(inputData, "i7", "i8")
        inputData = replace(inputData, "i8", "i9")
        inputData = replace(inputData, "w0", "w1")
        inputData = replace(inputData, "w1", "w2")
        inputData = replace(inputData, "w2", "w3")
        inputData = replace(inputData, "w3", "w4")
        inputData = replace(inputData, "w4", "w5")
        inputData = replace(inputData, "w5", "w6")
        inputData = replace(inputData, "w6", "w7")
        inputData = replace(inputData, "w7", "w8")
        inputData = replace(inputData, "w8", "w9")

        logThis("INFO", "Conversion completed...\n\n" & inputData)
        let exit = readLine(stdin)
        discard exit

    of "grid9ToDoc":
        logThis("INFO", "Converting Grid9 to Grid9 Docs")
        logThis("INFO", "Reading config file")

        if os.fileExists(replace(path, re".g9", ".toml")):
            let
                tomlPath = replace(path, re".g9", ".toml")
                config = toml_manager.getConfig(tomlPath)
            var minGrid9Ver = config[7]
            if minGrid9Ver > "2022.001":
                logThis("WARNING", "This conversion method is not supported for Grid9 versions below 2022.020, the output may not work as expected, continue? (y/n)")
                let answer = readLine(stdin)
                if answer != "y":
                    discard answer
                    return
                discard answer

        let inputFile = open(path, fmRead)
        var inputData = inputFile.readAll()
        inputFile.close()

        var outputData = ""
        
        # Replace all the characters with doc <span class="{type}">{content}</span>
        var currentChar = 0
        while currentChar < len(inputData):
            case inputData[currentChar]
            of 's':
                outputData &= "<span class='cm'>s" & inputData[currentChar + 1] & inputData[currentChar + 2] & "</span>"
                currentChar += 2
            of 'f':
                outputData &= "<span class='cm'>f" & inputData[currentChar + 1] & "</span>"
                currentChar += 1
            of 'a':
                outputData &= "<span class='cm'>a" & inputData[currentChar + 1] & "</span>"
                currentChar += 1
            of 'p':
                outputData &= "<span class='io'>p</span>"
            of 'q':
                outputData &= "<span class='io'>q" & inputData[currentChar + 1] & "</span>"
                currentChar += 1
            of 'i':
                outputData &= "<span class='log'>i" & inputData[currentChar + 1] & inputData[currentChar + 2] & inputData[currentChar + 3] & "</span>"
                currentChar += 3
            of 'w':
                outputData &= "<span class='log'>w" & inputData[currentChar + 1] & inputData[currentChar + 2] & inputData[currentChar + 3] & "</span>"
                currentChar += 3
            of '}':
                outputData &= "<span class='log'>}</span>"
            of ']':
                outputData &= "<span class='log'>]</span>"
            of 'e':
                outputData &= "<span class='log'>e</span>"
            of 'g':
                if inputData[currentChar + 1 ] == 'g':
                    outputData &= "<span class='io'>gg</span>"
                    currentChar += 1
                else:
                    outputData &= "<span class='msk'>g" & inputData[currentChar + 1] & inputData[currentChar + 2] & "</span>"
                    currentChar += 2
            of 'b':
                outputData &= "<span class='msk'>b"
                currentChar += 1
                while currentChar < len(inputData) and inputData[currentChar].isDigit():
                    outputData &= inputData[currentChar]
                    currentChar += 1
                outputData &= "</span>"
            of 'd':
                outputData &= "<span class='msk'>d"
                currentChar += 1
                while currentChar < len(inputData) and inputData[currentChar].isDigit():
                    outputData &= inputData[currentChar]
                    currentChar += 1
                outputData &= "</span>"
            of 't':
                outputData &= "<span class='msk'>t</span>"
            else:
                outputData &= "<span class='com'>" & inputData[currentChar] & "</span>"
            currentChar += 1

        logThis("INFO", "Conversion completed...\n\n" & outputData)
        let exit = readLine(stdin)
        discard exit
        
    else:
        logThis("ERROR", "Conversion not found; try any of the following: 'textToGrid9', 'grid9ToRetroGadget', 'grid9ToDoc'")

    clean()

proc main() =
    let args = docopt(doc, version = converterversion)

    if args["about"] or args["a"]:
        about()

    if args["version"] or args["v"]:
        version()

    if args["convert"] or args["c"]:
        convert($args["<path>"], $args["<conversion>"])

proc nonTerminal() =
    #Runs if no arguments are given
    echo "No arguments passed\n" & doc
    let exit = readLine(stdin)
    discard exit

when isMainModule:
    #Creates folders if they don't exist
    if not dirExists(subMainDir):
        createDir(subMainDir)
    if not dirExists(mainDir):
        createDir(mainDir)
    if not dirExists(logDir):
        createDir(logDir)

    #Checks if there are any arguments
    if len(os.commandLineParams()) > 0:
        if os.fileExists(os.commandLineParams()[0]) and len(os.commandLineParams()) == 1: convert(os.commandLineParams()[0], os.commandLineParams()[1])
        else: main()
    else: nonTerminal()
