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


def split2array(str, sep):
    if str == None or str.strip() == "":
        return []
    else:
        return str.strip().split(sep)


def split2intarray(str, sep):
    res = []
    if not str == None and not str.strip() == "" and \
       not str == None and not str.strip() == "" and \
       sep in str:
        for item in str.strip().split(sep):
            res.append(int(item))
    return res


def split2pair(str, sep):
    if sep in str:
        pair = str.split(sep)
        return (pair[0].strip(), pair[1].strip())
    else:
        return ()