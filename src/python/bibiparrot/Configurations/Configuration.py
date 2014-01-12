################################################################################
# Name     : Configuration.py                                                  #
# Brief    : define reading class Configuration for .cfg files.                #
#            e.g. ui.cfg                                                       #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

'''


'''

import sys, os , time, inspect, imp, platform, codecs
import logging
import ConfigParser

from ..Utils.utils import *
from BibiException import *
from ..Constants.constants import __configuration_file_encoding__, __project__, __version__
from configurations import LOGWIRE, log


###
##   Brief  : read file with first line encoding description.
#             e.g. "# Encoding=UTF-8"
#
#
def cfgEncoding(filename):
    if not os.path.exists(filename):
        raise BibiException("%s, filename=%s not exist!" %(funcname(), filename))
    try:
        fil = open(filename, 'r')
        line = fil.readline()
        fil.close()
    except Exception as e:
        raise BibiException("%s, filename=%s read line error!" %(funcname(), filename))

    if __configuration_file_encoding__ in line and "=" in line:
        enc = line.split("=")[1]
        enc = enc.strip()
        if LOGWIRE:
            log().debug("%s: file=%s, encoding=%s", funcname(), filename, enc)
        return enc
    return None


###
##   Brief  : Configuration class read encoded .cfg file.
#             First line will be "# Encoding=UTF-8"
#
#
class Configuration(object):
    ''' http://stackoverflow.com/questions/472000/python-slots '''
    __slots__=("CONF_FILE", "isConfLoaded", "config")

    def __init__(self):
        self.reset()

    def reset(self):
        self.CONF_FILE = ""
        self.isConfLoaded = False
        self.config = ConfigParser.ConfigParser()
        self.config.optionxform = str

    def asDict(self):
        result={}
        for item in self.__slots__:
            result[item] = getattr(self, item)
        return result

    def dump(self):
        config=self.asDict()
        if hasattr(platform, "python_implementation"):
            implementation = platform.python_implementation()
        else:
            implementation = "Jython" if os.name=="java" else "???"
        result=["Name: %s" % __project__,
                "Version: %s" % __version__,
                "Configuration file: %s" % os.path.abspath(os.path.split(inspect.getfile(self))[0]),
                "Python version: %s %s (%s, %s)" % (implementation, platform.python_version(), platform.system(), os.name),
                "Active configuration settings:"]
        for n, v in sorted(config.items()):
            result.append("%s = %s" % (n, v))
        return "{\n\t\t"+"\n\t\t".join(result)+"\n}"

    def loadConf(self):
        config_file = self.CONF_FILE
        if LOGWIRE:
           log().debug("%s: config_file=%s", funcname(),  config_file)
        if not os.path.isabs(config_file):
            if os.path.exists(os.path.join(directory(), self.CONF_FILE)):
                config_file = os.path.join(directory(), self.CONF_FILE)
        if os.path.exists(config_file):
            if LOGWIRE:
                log().debug("%s: config_file=%s", funcname(),  config_file)
            enc = cfgEncoding(config_file)
            if enc == None:
                self.config.read(config_file)
            else:
                fp = codecs.open(config_file, "r", enc)
                self.config.readfp(fp)
                fp.close()
            self.CONF_FILE = config_file
            self.isConfLoaded = True
        else:
            error = "Configuration file not exist: %s" % self.CONF_FILE
            raise BibiException(error)

    '''  @brief: read configuration file. '''
    def getConf(self, section, key):
        if not self.isConfLoaded:
            self.loadConf()
        try:
            val = self.config.get(section, key).strip()
        except ConfigParser.NoSectionError:
            val = None
        except ConfigParser.NoOptionError:
            val = None
        return val

    def reloadConf(self):
        self.isConfLoaded = False


    def setConf(self, section, key, value):
        if not self.isConfLoaded:
            self.loadConf()
        self.config.set(section, key, value)
        config_file = self.CONF_FILE
        if not os.path.isabs(config_file):
            config_file = os.path.join(directory(), self.CONF_FILE)
            self.CONF_FILE = config_file
        with open(self.CONF_FILE, 'wb') as configfile:
            self.config.write(configfile)
            configfile.close()
        return value
