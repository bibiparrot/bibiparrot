################################################################################
# Name     : RichTextFormatTextHandler.py                                      #
# Brief    : Define rtf handler for wx.richtext.RichTextCtrl                   #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/                                 #
#                                 wx.richtext.RichTextFileHandler-class.html   #
# Url      : http://xoomer.virgilio.it/infinity77/wxPython/                    #
#                              richtext/wx.richtext.RichTextFileHandler.html   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


from ...bibiparrot.Constants.constants import __required_wx_version__

import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.richtext
import wx._richtext
import wx.html
import wx.aui

from encodings.aliases import aliases

def findEncodings(q):
    return [(k,v) for k, v in aliases.items() if q in k or q in v]

class RichTextFormatTextHandler(wx.richtext.RichTextPlainTextHandler):
    def __init__(self, *args, **kwargs):
        self.name = 'RTF'
        self.ext = 'rtf'
        self.enc = 'utf-8'
        ###  flags is permission http://www.dartmouth.edu/~rc/help/faq/permissions.html ###
        self.flags = 0
        wx.richtext.RichTextPlainTextHandler.__init__(self, name =self.name,
                                                           ext=self.ext,
                                                           type=wx.richtext.RICHTEXT_TYPE_RTF)
    	def CanHandle(self, filename):
            ext = os.path.splitext(filename)[-1].strip().lower()
            return ext == self.ext

        def CanLoad(self):
            return True
        def CanSave(self):
            return True
        def GetEncoding(self):
            return self.enc
        def GetExtension(self):
            return self.ext
        def GetFlags(self):
            return self.flags

        def GetName(self):
            return self.name

        def	GetType(self):
            return wx.richtext.RICHTEXT_TYPE_RTF

        def IsVisible(self):
            return True

        def SetEncoding(self, encoding):
            if encoding is not None and len(findEncodings(encoding)) > 0:
                self.enc = encoding

        def SetExtension(self, ext):
            self.ext = ext
        def SetFlags(self, flags):
            self.flags = flags
        def SetName(self, name):
            self.name = name
        def SetType(self, type):
            pass
        def SetVisible(self, visible):
            pass

        def	LoadFile(self, buffer, filename):
            print "LoadFile"
            isLoaded = False
            return isLoaded
        def	LoadStream(self, buffer, stream):
            isLoaded = False
            return isLoaded
        def	SaveFile(self, buffer, filename):
            isSaved = False
            return isSaved
        def	SaveStream(self, buffer, stream):
            isSaved = False
            return isSaved
