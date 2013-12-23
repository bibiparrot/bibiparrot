################################################################################
# Name     : utils.py                                                          #
# Brief    : Import the required base modules needed for launching Bibiparrot  #
#                                                                              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################




import sys, os , time, inspect, imp, platform, logging, traceback
import wx
import ConfigParser


##
# @url: http://stackoverflow.com/questions/1095543/get-name-of-calling-functions-module-in-python
##
def directory():
    filename = os.path.abspath(inspect.getfile(inspect.stack()[1][0]))
    path = os.path.dirname(filename)
    return path


def funcname():
    stack = traceback.extract_stack()
    filename, lineno, name, text = stack[-2]
    return name