################################################################################
# Name     : BiBiParrotFormatTextHandler.py                                    #
# Brief    : Define .bbp (zipped xml) handler for wx.richtext.RichTextCtrl     #
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
import zipfile

from ...bibiparrot.Configurations.configurations import *

from encodings.aliases import aliases

def findEncodings(q):
    return [(k,v) for k, v in aliases.items() if q in k or q in v]

class BiBiParrotFormatTextHandler(wx.richtext.RichTextXMLHandler):
    ### [RICHTEXT_TYPE_ANY,RICHTEXT_TYPE_TEXT,RICHTEXT_TYPE_XML,RICHTEXT_TYPE_RTF,RICHTEXT_TYPE_PDF] ###
    ### [0, 1, 2, 3, 4, 5 ] ###
    RICHTEXT_TYPE_BBP = wx.richtext.RICHTEXT_TYPE_TEXT + 6
    def __init__(self, *args, **kwargs):
        self.name = 'Bibi Parrot'
        self.ext = 'bbp'
        self.enc = 'utf-8'
        ###  flags is permission http://www.dartmouth.edu/~rc/help/faq/permissions.html ###
        self.flags = 0
        wx.richtext.RichTextXMLHandler.__init__(self, name =self.name,
                                                           ext=self.ext,
                                                           type=BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP)

        ### add handler into configurations.rtchandlers ###
        rtchandlers[BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP] = BiBiParrotFormatTextHandler
        pass
    #
    # def CanHandle(self, filename):
    #     ext = os.path.splitext(filename)[-1].strip().lower()
    #     return ext == self.ext
    #
    # def CanLoad(self):
    #     return True
    #
    # def CanSave(self):
    #     return True
    #
    # def GetEncoding(self):
    #     return self.enc
    #
    # def GetExtension(self):
    #     return self.ext
    #
    # def GetFlags(self):
    #     return self.flags
    #
    # def GetName(self):
    #     return self.name
    #
    # def	GetType(self):
    #     return BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP
    #
    # def IsVisible(self):
    #     return True
    #
    # def SetEncoding(self, encoding):
    #     if encoding is not None and len(findEncodings(encoding)) > 0:
    #         self.enc = encoding
    #
    # def SetExtension(self, ext):
    #     self.ext = ext
    # def SetFlags(self, flags):
    #     self.flags = flags
    # def SetName(self, name):
    #     self.name = name

    # def SetType(self, type):
    #     pass
    # def SetVisible(self, visible):
    #     pass

    def	LoadFile(self, buffer, filename):
        print "LoadFile=%s"%filename
        # isLoaded = False
        xml = filename + ".xml~"
        fname = os.path.basename(xml)
        try:
            fp = open(xml,'w')
            zf = None
            try:
                zf = zipfile.ZipFile(filename,'r')
                fp.write(zf.read(fname), compress_type=zipfile.ZIP_DEFLATED)
            except zipfile.BadZipfile as ex:
                wx.MessageBox("This is NOT Valid Bibi Parrot file.", "File Error!")
                return False
            finally:
                if zf is not None:
                    zf.close()
            # print xml
        finally:
            if fp is not None:
                fp.close()
        if os.path.exists(xml):
            ret = wx.richtext.RichTextXMLHandler.LoadFile(self, buffer, xml)
            os.remove(xml)
            return ret
        else:
            return False

        # return isLoaded

    # def	LoadStream(self, buffer, stream):
    #     # isLoaded = False
    #     # return isLoaded
    #     return wx.richtext.RichTextXMLHandler.LoadStream(self, buffer, stream)

    def	SaveFile(self, buffer, filename):
        # isSaved = False
        # return isSaved
        # dir = os.path.dirname(filename)
        xml = filename + ".xml~"
        try:
            ret = wx.richtext.RichTextXMLHandler.SaveFile(self, buffer, xml)
            zf = zipfile.ZipFile(filename, "w")
            try:
                zf.write(xml, arcname=os.path.basename(xml))
            finally:
                zf.close()
        finally:
            if os.path.exists(xml):
                os.remove(xml)
        return ret


    # def	SaveStream(self, buffer, stream):
    #     # isSaved = False
    #     # return isSaved
    #     return wx.richtext.RichTextXMLHandler.SaveStream(self, buffer, stream)
