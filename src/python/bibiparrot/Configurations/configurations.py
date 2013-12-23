'''


'''

import sys, os , time, inspect, imp, platform
import logging
import wx
import ConfigParser

from ..Utils.utils import *
from BibiException import *
from ..Constants.constants import __default__logging_file__, __default__ui__config__

logging.basicConfig(filename=__default__logging_file__, filemode='w', level=logging.DEBUG)
# handler = logging.handlers.RotatingFileHandler(__default__logging_file__, maxBytes=20, backupCount=5)
# log.setLevel(logging.DEBUG)
# log.addHandler(handler)
LOGWIRE = True


logs = {}

def log():
    stack = traceback.extract_stack()
    filename, lineno, name, text = stack[-2]
    key = os.path.splitext(os.path.basename(filename))[0]
    if not logs.has_key(key):
        logs[key] = logging.getLogger(key)
    return logs[key]


confs = {}
def uiconf():
    key = funcname()
    if not confs.has_key(key):
        from Configuration import Configuration
        confs[key] = Configuration()
        confs[key].CONF_FILE = __default__ui__config__
    return confs[key]
