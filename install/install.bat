set key=qcodecloudagent
reg add HKCR\%key% /ve /d "QCode Cloud Agent"
reg add HKCR\%key% /v "URL Protocol" /d ""
reg add HKCR\%key%\shell
reg add HKCR\%key%\shell\open
reg add HKCR\%key%\shell\open\command /ve /d "\"C:\Program Files\QCodeCloudAgent\protocol.exe\" \"%%1\""
