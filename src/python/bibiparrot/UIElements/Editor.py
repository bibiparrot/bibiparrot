
import sys, os , time, inspect, imp, platform, logging
import wx
import wx.grid
import wx.html
import wx.aui

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.UIElements.UIElement import UIElement



class Editor(wx.richtext.RichTextCtrl):
    def __init__(self):
        pass