#!/bin/sh


################################################################################
# Name     : testbibiparrot.sh                                                 #
# Brief    : Running the unittest module                                       #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

#FILE=$(readlink -f "$0")
#HOME=$(dirname $FILE)
HOME="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $HOME/..
#python -m python.testbibiparrot.Configurations.TestBibiException
#python -m python.testbibiparrot.Configurations.TestConfiguration
#python -m python.testbibiparrot.UIElements.TestUIElement
#python -m python.testbibiparrot.UIElements.TestMainMenu
#python -m python.testbibiparrot.UIElements.TestMainToolbar
#python -m python.testbibiparrot.UIElements.TestEditControl
#python -m python.testbibiparrot.UIElements.TestEditor
#python -m python.testbibiparrot.UIElements.TestMainTabs
#python -m python.testbibiparrot.UIElements.TestMainStatusbar
python -m python.testbibiparrot.UIElements.TestRepeater
#python -m python.testbibiparrot.audioUtils.TestMediaPlayControl
cd $HOME
