################################################################################
# Name     : Editor.py                                                         #
# Brief    : #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.ToolBar-class.html            #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################



from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.UIElements.UIElement import UIElement

import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.grid
import wx.html
import wx.aui




class Editor(wx.richtext.RichTextCtrl):
    def __init__(self):
        pass