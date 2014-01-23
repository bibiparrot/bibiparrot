@echo off
REM @TIME: 2014-01-07
REM @AUTHOR: Chunqi SHI (diligence.cs@gmail.com)
REM @INF: start test the modules of python
REM


REM ############################################################################
REM # Name     : testbibiparrot.bat                                            #
REM # Brief    : Running the unittest module                                   #
REM #                                                                          #
REM # Url      :                                                               #
REM # Author   : Chunqi SHI <diligence.cs@gmail.com>                           #
REM # Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>    #
REM ############################################################################


set HOME=%~dp0
cd %HOME%/..
python -m python.testbibiparrot.Configurations.TestBibiException
python -m python.testbibiparrot.Configurations.TestConfiguration
REM python -m python.testbibiparrot.UIElements.TestMainFrame
REM python -m python.testbibiparrot.UIElements.TestUIElement
REM python -m python.testbibiparrot.UIElements.TestMainMenu
REM python -m python.testbibiparrot.UIElements.TestMainToolbar
REM python -m python.testbibiparrot.UIElements.TestEditor
REM python -m python.testbibiparrot.UIElements.TestMainTabs
REM python -m python.testbibiparrot.UIElements.TestMainStatusbar
python -m python.testbibiparrot.UIElements.TestRepeater
REM python -m python.testbibiparrot.audioUtils.TestMediaPlayControl
cd %HOME%
