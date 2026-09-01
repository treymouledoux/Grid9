---@diagnostic disable: param-type-mismatch

-- Input Grid9 Code here
Code = "i1=0f9f8f4qsa0f8f7qsa0f9f7f6qsa0f9f7f6qsa0f5qsa0qsa0f6f5qsa0f5qsa0f9f8f5qsa0f9f7f6qsa0f9f7qspa0}"

-- Grid9 Vars
local spriteFont = gdt.ROM.System.SpriteSheets["StandardFont"]
NumberToBool = { [1]=true, [0]=false }
Grid = {0,0,0,0,0,0,0,0,0}
LoopPos = -1
Queue = ""
ScreenContent = ""
Glyphs = {
    ["000000000"] = " ",
    ["000000001"] = "\n",
    ["000000010"] = "a",
    ["000000011"] = "b",
    ["000000100"] = "c",
    ["000000101"] = "d",
    ["000000110"] = "e",
    ["000000111"] = "f",
    ["000001000"] = "g",
    ["000001001"] = "h",
    ["000001010"] = "i",
    ["000001011"] = "j",
    ["000001100"] = "k",
    ["000001101"] = "l",
    ["000001110"] = "m",
    ["000001111"] = "n",
    ["000010000"] = "o",
    ["000010001"] = "p",
    ["000010010"] = "q",
    ["000010011"] = "r",
    ["000010100"] = "s",
    ["000010101"] = "t",
    ["000010110"] = "u",
    ["000010111"] = "v",
    ["000011000"] = "w",
    ["000011001"] = "x",
    ["000011010"] = "y",
    ["000011011"] = "z",
    ["000011100"] = "A",
    ["000011101"] = "B",
    ["000011110"] = "C",
    ["000011111"] = "D",
    ["000100000"] = "E",
    ["000100001"] = "F",
    ["000100010"] = "G",
    ["000100011"] = "H",
    ["000100100"] = "I",
    ["000100101"] = "J",
    ["000100110"] = "K",
    ["000100111"] = "L",
    ["000101000"] = "M",
    ["000101001"] = "N",
    ["000101010"] = "O",
    ["000101011"] = "P",
    ["000101100"] = "Q",
    ["000101101"] = "R",
    ["000101110"] = "S",
    ["000101111"] = "T",
    ["000110000"] = "U",
    ["000110001"] = "V",
    ["000110010"] = "W",
    ["000110011"] = "X",
    ["000110100"] = "Y",
    ["000110101"] = "Z",
    ["000110110"] = "0",
    ["000110111"] = "1",
    ["000111000"] = "2",
    ["000111001"] = "3",
    ["000111010"] = "4",
    ["000111011"] = "5",
    ["000111100"] = "6",
    ["000111101"] = "7",
    ["000111110"] = "8",
    ["000111111"] = "9",
    ["001000000"] = ".",
    ["001000001"] = ",",
    ["001000010"] = "?",
    ["001000011"] = "!",
    ["001000100"] = ":",
    ["001000101"] = ";",
    ["001000110"] = "-",
    ["001000111"] = "_",
    ["001001000"] = "'",
    ["001001001"] = "\\",
    ["001001010"] = "(",
    ["001001011"] = ")",
    ["001001100"] = "[",
    ["001001101"] = "]",
    ["001001110"] = "{",
    ["001001111"] = "}",
    ["001010000"] = "=",
    ["001010001"] = "+",
    ["001010010"] = "*",
    ["001010011"] = "/",
    ["001010100"] = "|",
    ["001010101"] = "&",
    ["001010110"] = "^",
    ["001010111"] = "%",
    ["001011000"] = "$",
    ["001011001"] = "@",
    ["001011010"] = "#",
    ["001011011"] = "~",
    ["001011100"] = "`"
}

-- Update function is repeated every time tick
function update()
	if (gdt.LedButton1.ButtonDown) then
        gdt.LedButton1.LedState = true
        gdt["Led9"].State = true
        ScreenContent = ""

		gdt.LedButton1.LedState = not gdt.LedButton1.LedState
		local c_index = 0
		while (c_index < #Code + 1) do
			if (Code:sub(c_index, c_index) == "f") then
				if (Grid[tonumber(Code:sub(c_index+1, c_index+1))] == 0) then
                    -- replace value in grid with 1
                    Grid[tonumber(Code:sub(c_index+1, c_index+1))] = 1
                    gdt["Led" .. Code:sub(c_index+1, c_index+1) - 1].State = true
				else
                    -- replace value in grid with 0
                    Grid[tonumber(Code:sub(c_index+1, c_index+1))] = 0
                    gdt["Led" .. Code:sub(c_index+1, c_index+1) - 1].State = false
				end
			end
            if (Code:sub(c_index, c_index) == "s") then
                -- Set value to specific value
                Grid[tonumber(Code:sub(c_index+1, c_index+1))] = tonumber(Code:sub(c_index+2, c_index+2))
                gdt["Led" .. Code:sub(c_index+1, c_index+1) - 1].State = NumberToBool[tonumber(Code:sub(c_index+2, c_index+2))]
            end
            if (Code:sub(c_index, c_index) == "a") then
                -- All values to specific value
                for i = 1, #Grid do
                    Grid[i] = tonumber(Code:sub(c_index+1, c_index+1))
                    gdt["Led" .. i - 1].State = NumberToBool[tonumber(Code:sub(c_index+1, c_index+1))]
                end
            end
            if (Code:sub(c_index, c_index) == "q") then
                if (Code:sub(c_index+1, c_index+1) == "s") then
                    -- Save to queue
                    local glyph = table.concat(Grid)
                    Queue = Queue .. Glyphs[tostring(glyph)]
                    c_index += 1
                else
                    -- Clear queue
                    Queue = ""
                    c_index += 1
                end
            end 
			if (Code:sub(c_index, c_index) == "p") then
                if (Queue == "") then
                    local glyph = table.concat(Grid)
                    --log(Glyphs[tostring(glyph)])
                    ScreenContent = ScreenContent .. "\n" .. Glyphs[tostring(glyph)]
                    RenderOutput()
                else
                    --log(Queue)
                    ScreenContent = ScreenContent .. "\n" .. Queue
                    Queue = ""
                    RenderOutput()
                end
			end
            if (Code:sub(c_index, c_index) == "b") then
                c_index -= Code:sub(c_index+1, c_index+1)
            end
            if (Code:sub(c_index, c_index) == "w") then
                LoopPos = c_index
            end
            if (Code:sub(c_index, c_index) == "]") then
                if LoopPos ~= -1 then
                    if (Code:sub(LoopPos+2, LoopPos+2) == "=") then
                        if Grid[tonumber(Code:sub(LoopPos+1, LoopPos+1))] == tonumber(Code:sub(LoopPos+3, LoopPos+3)) then
                            c_index = LoopPos
                        else
                            LoopPos = -1
                        end
                    elseif (Code:sub(LoopPos+2, LoopPos+2) == "!") then
                        if Grid[tonumber(Code:sub(LoopPos+1, LoopPos+1))] ~= tonumber(Code:sub(LoopPos+3, LoopPos+3)) then
                            c_index = LoopPos
                        else
                            LoopPos = -1
                        end
                    end
                end
            end
            if (Code:sub(c_index, c_index) == "i") then
                if (Code:sub(c_index+2, c_index+2) == "=") then
                    if Grid[tonumber(Code:sub(c_index+1, c_index+1))] == tonumber(Code:sub(c_index+3, c_index+3)) then
                        -- do nothing
                    else
                        while (Code:sub(c_index, c_index) ~= "}") do
                            c_index += 1
                            if (c_index > #Code) then
                                log("Error: No closing bracket found for if statement")
                                break
                            end
                        end
                    end
                elseif (Code:sub(c_index+2, c_index+2) == "!") then
                    if Grid[tonumber(Code:sub(c_index+1, c_index+1))] ~= tonumber(Code:sub(c_index+3, c_index+3)) then
                        -- do nothing
                    else
                        while (Code:sub(c_index, c_index) ~= "}") do
                            c_index += 1
                            if (c_index > #Code) then
                                log("Error: No closing bracket found for if statement")
                                break
                            end
                        end
                    end
                end
            end
            if (Code:sub(c_index, c_index) == "x") then
                while (Code:sub(c_index, c_index) ~= "}" or Code:sub(c_index, c_index) ~= "]") do
                    c_index += 1
                    if (c_index > #Code) then
                        log("Error: Depth at 0 no way to climb higher")
                        break
                    end
                end
            end
            if (Code:sub(c_index, c_index) == "t") then
                break
            end
            -- Print grid to console every frame
            --log(table.concat(Grid))
			c_index += 1
		end
        -- Print the grid to the console when code is finished
        --log(table.concat(Grid))
        gdt.LedButton1.LedState = false
        gdt["Led9"].State = false
	end

    RenderOutput()

end

function RenderOutput()
    local scrollHeight = math.floor (gdt.Slider0.Value+0.5)
    local linelettersC = 0
    local lineOutput = ""
    local lineHeight = 0
    local lineLength = 12

    gdt.VideoChip0.Clear(gdt.VideoChip0, color.black)

    for i = 1, #ScreenContent do
        linelettersC += 1
        lineOutput = lineOutput .. ScreenContent:sub(i, i)

        -- Displays only 12 letters per line
        if (linelettersC > lineLength) then
            -- Check if the line output is too long
            if (lineOutput:len() > lineLength) then
                lineOutput = lineOutput:sub(1, lineLength)
            end
            gdt.VideoChip0:DrawText(vec2(1, lineHeight+(-scrollHeight)),spriteFont,lineOutput,color.white,color.black)
            linelettersC = 0
            lineHeight += 10
            lineOutput = ""

            -- If it is \n then skip a line
            if (ScreenContent:sub(i, i) == "\n") then
                linelettersC = 0
                lineHeight += 10
                lineOutput = ""
            end
        end
    end
    gdt.VideoChip0:DrawText(vec2(1, lineHeight+(-scrollHeight)),spriteFont,lineOutput,color.white,color.black)
end