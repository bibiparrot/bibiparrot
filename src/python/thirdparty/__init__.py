################################################################################
# Name     : __init__.py                                                       #
# Brief    : Import the required third party modules needed for                #
#            launching Bibiparrot into the namespace.                          #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

__all__ = ["mutagen"]

import sys,os,inspect
for mo in __all__:
    path = os.path.join(os.path.abspath(os.path.dirname(inspect.getfile(inspect.stack()[0][0]))), mo)
    sys.path.append(path)