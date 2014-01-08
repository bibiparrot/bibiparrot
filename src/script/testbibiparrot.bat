@echo off

set HOME=%~dp0
cd %HOME%/..
python -m python.testbibiparrot.Configurations.TestBibiException
python -m python.testbibiparrot.Configurations.TestConfiguration
REM python -m python.testbibiparrot.UIElements.TestMainFrame
REM python -m python.testbibiparrot.UIElements.TestUIElement
REM python -m python.testbibiparrot.UIElements.TestMainMenu
python -m python.testbibiparrot.UIElements.TestMainToolbar
python -m python.testbibiparrot.UIElements.TestEditor
cd %HOME%
