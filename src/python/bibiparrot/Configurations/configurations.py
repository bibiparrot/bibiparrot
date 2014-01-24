################################################################################
# Name     : configurations.py                                                 #
# Brief    : define the necessary settings for the bibiparrot.                 #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

'''


'''

import sys, os , time, inspect, imp, platform
import logging
import ConfigParser

from ..Utils.utils import *
from BibiException import *
from ..Constants.constants import __default_logging_file__, __default_ui_config__


logging.basicConfig(filename=__default_logging_file__, filemode='w', level=logging.DEBUG)
# handler = logging.handlers.RotatingFileHandler(__default_logging_file__, maxBytes=20, backupCount=5)
# log.setLevel(logging.DEBUG)
# log.addHandler(handler)
LOGWIRE = True

##
#  logger dictionary
#
logs = {}

###
##   Brief  : return the reference of new logger,
#             logger's name is the python file name.
#             all the logs will be cashed into logs dictionary.
#

def log():
    stack = traceback.extract_stack()
    filename, lineno, name, text = stack[-2]
    key = os.path.splitext(os.path.basename(filename))[0]
    return logs.get(key, logging.getLogger(key))

##
#  configurations dictionary
#
confs = {}

###
##   Brief  : return the reference of new ui configuration.
#
#


def uiconf(key):
    conf = confs.get(key, None)
    if conf is None:
        from Configuration import Configuration
        conf = Configuration()
        conf.CONF_FILE = __default_ui_config__
        confs[key] = conf
    return conf

# @DeprecationWarning
# def uiconfig():
#     key = funcname()
#     if not confs.has_key(key):
#
#         confs[key] = Configuration()
#         confs[key].CONF_FILE = __default_ui_config__
#     return confs[key]


##
#  self-defined id {[id:(wxId, name),]}
#
eventids = {}

##
#  self-defined rich text control handlers
#
rtchandlers = {}

##
#  self-defined controllers
#
selfctrls = {}


##
#  self-defined dialogs
#
selfdialogs = {}

