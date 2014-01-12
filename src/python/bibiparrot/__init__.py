################################################################################
# Name     : __init__.py                                                       #
# Brief    : Import the required base modules needed for launching Bibiparrot  #
#            into the namespace.                                               #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

__all__ = ['UIElements']


import sys, os , time, inspect, imp, platform, logging
import ConfigParser
import cPickle as pickle


if sys.version_info < (2, 6):
    import warnings
    warnings.warn("This BibiParrot Version is Unsupported on Python Versions Older Than 2.6", ImportWarning)





