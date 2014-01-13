#!/usr/bin/env python

################################################################################
# Name     : testbibiparrot.py                                                 #
# Brief    : running test from python file                                     #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import os
from subprocess import call

def test_module(name):
    '''
        1. change directory to module root $ProjectHome/src;
        2. set environment for using 32 bit python version;
        3. print current directory
        4. execute python module
    '''
    os.chdir(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    print "Current Directory:", os.getcwd()
    call(["python","-m", name])

if __name__ == '__main__':
    test_module("python.testbibiparrot.UIElements.TestMainToolbar")
    test_module("python.testbibiparrot.UIElements.TestEditor")