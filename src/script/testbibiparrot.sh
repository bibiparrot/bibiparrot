#!/bin/sh


#FILE=$(readlink -f "$0")
#HOME=$(dirname $FILE)
HOME="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $HOME/..
python -m python.testbibiparrot.Configurations.TestBibiException
python -m python.testbibiparrot.Configurations.TestConfiguration
python -m python.testbibiparrot.UIElements.TestMainFrame
cd $HOME
