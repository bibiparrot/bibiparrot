################################################################################
# Name     : EditorControl.py                                                  #
# Brief    : Define the base functions to control a rtf file                   #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################



from ...bibiparrot.Constants.constants import __required_wx_version__

import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.grid
import wx.html
import wx.aui
import wx.richtext

from ..Configurations import configurations
from ..Constants import constants
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.Configurations.configurations import *
from BiBiParrotFormatTextHandler import BiBiParrotFormatTextHandler






class EditControl(wx.richtext.RichTextCtrl):
    def __init__(self, parent, *args, **kwargs):
        wx.richtext.RichTextCtrl.__init__(self, parent, style=wx.VSCROLL|wx.HSCROLL|wx.NO_BORDER, *args, **kwargs)
        wx.CallAfter(self.SetFocus)
        self.currDir = ""
        self.textAttr = wx.richtext.RichTextAttr()
        self.AddRTCHandlers()
        self.Freeze()
        self.BeginSuppressUndo()

        self.BeginParagraphSpacing(0, 20)

        self.BeginAlignment(wx.richtext.TEXT_ALIGNMENT_CENTRE)
        self.BeginBold()

        self.BeginFontSize(14)
        self.WriteText("Welcome to wxRichTextCtrl, a wxWidgets control for editing and presenting styled text and images")
        self.EndFontSize()
        self.Newline()
        self.EndSuppressUndo()
        self.Thaw()
        pass

    def AddRTCHandlers(self):
        if wx.richtext.RichTextBuffer.FindHandlerByType(BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP) is not None:
            return
        wx.richtext.RichTextBuffer.AddHandler(BiBiParrotFormatTextHandler())
        wx.richtext.RichTextBuffer.AddHandler(wx.richtext.RichTextHTMLHandler())
        wx.richtext.RichTextBuffer.AddHandler(wx.richtext.RichTextXMLHandler())
        ### Add the rtf text handler ###
        # wx.richtext.RichTextBuffer.AddHandler(wx.richtext.RichTextHTMLHandler(name="Other XML",
        #                                                    ext="html",
        #                                                    type=wx.richtext.RICHTEXT_TYPE_HTML))

        # This is needed for the view as HTML option since we tell it
        # to store the images in the memory file system.
        wx.FileSystem.AddHandler(wx.MemoryFSHandler())



    def SetFontStyle(self, fontColor = None, fontBgColor = None, fontFace = None, fontSize = None,
                     fontBold = None, fontItalic = None, fontUnderline = None):
      if fontColor:
         self.textAttr.SetTextColour(fontColor)
      if fontBgColor:
         self.textAttr.SetBackgroundColour(fontBgColor)
      if fontFace:
         self.textAttr.SetFontFaceName(fontFace)
      if fontSize:
         self.textAttr.SetFontSize(fontSize)
      if fontBold != None:
         if fontBold:
            self.textAttr.SetFontWeight(wx.FONTWEIGHT_BOLD)
         else:
            self.textAttr.SetFontWeight(wx.FONTWEIGHT_NORMAL)
      if fontItalic != None:
         if fontItalic:
            self.textAttr.SetFontStyle(wx.FONTSTYLE_ITALIC)
         else:
            self.textAttr.SetFontStyle(wx.FONTSTYLE_NORMAL)
      if fontUnderline != None:
         if fontUnderline:
            self.textAttr.SetFontUnderlined(True)
         else:
            self.textAttr.SetFontUnderlined(False)
      self.SetDefaultStyle(self.textAttr)

    def OnURL(self, evt):
        wx.MessageBox(evt.GetString(), "URL Clicked")

    def LoadFile(self, path, type, *args, **kwargs):
        ### has self-defined types, needing self-defined handler ###
        if rtchandlers.has_key(type):
            # print file
            self.Freeze()
            self.BeginSuppressUndo()
            # Clear the Control AND the default text attributes
            self.Clear()
            handler = rtchandlers[type]()
            ret = handler.LoadFile(self.GetBuffer(), path)
            self.EndSuppressUndo()
            self.Thaw()
            return ret
        else:
            return wx.richtext.RichTextCtrl.LoadFile(self, path, type, *args, **kwargs)

    def OnFileOpen(self, evt):
        # This gives us a string suitable for the file dialog based on
        # the file handlers that are loaded
        # wildcard, types = wx.richtext.RichTextBuffer.GetExtWildcard(save=False)
        print "OnFileOpen"
        wildcard = 'Bibi Parrot (*.bbp)|*.bbp|Text (*.txt)|*.txt|XML (*.xml)|*.xml'
        types = [BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP,
                 wx.richtext.RICHTEXT_TYPE_TEXT,
                 wx.richtext.RICHTEXT_TYPE_XML]
        dlg = wx.FileDialog(self, "Choose a filename",
                            wildcard=wildcard,
                            style=wx.OPEN,
                            defaultDir=self.currDir)

        if dlg.ShowModal() == wx.ID_OK:
            ''' Remember last directory'''
            self.currDir = dlg.GetDirectory()
            path = dlg.GetPath()
            if path:
                fileType = types[dlg.GetFilterIndex()]
                # print path, fileType
                self.LoadFile(path, fileType)
                self.SetFilename(path)
        dlg.Destroy()


    def OnFileSave(self, evt):
        print self.GetFilename()
        if not self.GetFilename():
            self.OnFileSaveAs(evt)
            return
        self.OnFileSaveAs(evt)

    def SaveFile(self, path, type, *args, **kwargs):
        if rtchandlers.has_key(type):
            handler = rtchandlers[type]()
            ret = handler.SaveFile(self.GetBuffer(), path)
            return ret

        else:
            return wx.richtext.RichTextCtrl.SaveFile(self, path, type, *args, **kwargs)


    def OnFileSaveAs(self, evt):
        # wildcard, types = wx.richtext.RichTextBuffer.GetExtWildcard(save=True)
        wildcard = 'Bibi Parrot (*.bbp)|*.bbp|Text (*.txt)|*.txt|XML (*.xml)|*.xml|HTML (*.html)|*.html'
        types = [BiBiParrotFormatTextHandler.RICHTEXT_TYPE_BBP,
                 wx.richtext.RICHTEXT_TYPE_TEXT,
                 wx.richtext.RICHTEXT_TYPE_XML,
                 wx.richtext.RICHTEXT_TYPE_HTML]
        dlg = wx.FileDialog(self, "Choose a filename",
                            wildcard=wildcard,
                            style=wx.SAVE)
        if dlg.ShowModal() == wx.ID_OK:
            path = dlg.GetPath()
            if path:
                fileType = types[dlg.GetFilterIndex()]
                ext = wx.richtext.RichTextBuffer.FindHandlerByType(fileType).GetExtension()
                if not path.endswith(ext):
                    path += '.' + ext
                self.SaveFile(path, fileType)
        dlg.Destroy()


    def OnFileViewHTML(self, evt):
        # Get an instance of the html file handler, use it to save the
        # document to a StringIO stream, and then display the
        # resulting html text in a dialog with a HtmlWindow.
        handler = wx.richtext.RichTextHTMLHandler()
        handler.SetFlags(wx.richtext.RICHTEXT_HANDLER_SAVE_IMAGES_TO_MEMORY)
        handler.SetFontSizeMapping([7,9,11,12,14,22,100])

        import cStringIO
        stream = cStringIO.StringIO()
        if not handler.SaveStream(self.GetBuffer(), stream):
            return

        import wx.html
        dlg = wx.Dialog(self, title="HTML", style=wx.DEFAULT_DIALOG_STYLE|wx.RESIZE_BORDER)
        html = wx.html.HtmlWindow(dlg, size=(500,400), style=wx.BORDER_SUNKEN)
        html.SetPage(stream.getvalue())
        btn = wx.Button(dlg, wx.ID_CANCEL)
        sizer = wx.BoxSizer(wx.VERTICAL)
        sizer.Add(html, 1, wx.ALL|wx.EXPAND, 5)
        sizer.Add(btn, 0, wx.ALL|wx.CENTER, 10)
        dlg.SetSizer(sizer)
        sizer.Fit(dlg)

        dlg.ShowModal()

        handler.DeleteTemporaryImages()


    def OnFileExit(self, evt):
        self.Close(True)

    ### Find & Replace ###
    def OnShowFind(self, evt):
        self.DisableButtons()
        data = wx.FindReplaceData()
        dlg = wx.FindReplaceDialog(self, data, "Find")
        dlg.data = data  # save a reference to it...
        dlg.Show(True)


    def OnShowFindReplace(self, evt):
        self.DisableButtons()
        data = wx.FindReplaceData()
        dlg = wx.FindReplaceDialog(self, data, "Find & Replace", wx.FR_REPLACEDIALOG)
        dlg.data = data  # save a reference to it...
        dlg.Show(True)


    def OnFind(self, evt):
        map = {
            wx.wxEVT_COMMAND_FIND : "FIND",
            wx.wxEVT_COMMAND_FIND_NEXT : "FIND_NEXT",
            wx.wxEVT_COMMAND_FIND_REPLACE : "REPLACE",
            wx.wxEVT_COMMAND_FIND_REPLACE_ALL : "REPLACE_ALL",
            }

        et = evt.GetEventType()

        if et in map:
            evtType = map[et]
        else:
            evtType = "**Unknown Event Type**"

        if et in [wx.wxEVT_COMMAND_FIND_REPLACE, wx.wxEVT_COMMAND_FIND_REPLACE_ALL]:
            replaceTxt = "Replace text: %s" % evt.GetReplaceString()
        else:
            replaceTxt = ""

        # self.log.write("%s -- Find text: %s   Replace text: %s  Flags: %d  \n" %
        #                (evtType, evt.GetFindString(), replaceTxt, evt.GetFlags()))


    def OnFindClose(self, evt):
        # self.log.write("FindReplaceDialog closing...\n")
        evt.GetDialog().Destroy()
        # self.EnableButtons()

    # def EnableButtons(self):
    #     self.fbtn.Enable()
    #     self.frbtn.Enable()
    #
    # def DisableButtons(self):
    #     self.fbtn.Disable()
    #     self.frbtn.Disable()

    def OnSearchBox(self,evt):
        # # Create controls
        # sb = wx.StaticBox(self, -1, "Options")
        # searchBtnOpt = wx.CheckBox(self, -1, "Search button")
        # searchBtnOpt.SetValue(True)
        # cancelBtnOpt = wx.CheckBox(self, -1, "Cancel button")
        # menuBtnOpt   = wx.CheckBox(self, -1, "Search menu")
        #
        # self.search = wx.SearchCtrl(self, size=(200,-1), style=wx.TE_PROCESS_ENTER)
        #
        # # Setup the layout
        # box = wx.StaticBoxSizer(sb, wx.VERTICAL)
        # box.Add(searchBtnOpt, 0, wx.ALL, 5)
        # box.Add(cancelBtnOpt, 0, wx.ALL, 5)
        # box.Add(menuBtnOpt,   0, wx.ALL, 5)
        #
        # sizer = wx.BoxSizer(wx.HORIZONTAL)
        # sizer.Add(box, 0, wx.ALL, 15)
        # sizer.Add((15,15))
        # sizer.Add(self.search, 0, wx.ALL, 15)
        pass

    ### Bold, Italic, Underline, StrikeThrough ###
    def OnBold(self, evt):
        self.ApplyBoldToSelection()

    def OnItalic(self, evt):
        self.ApplyItalicToSelection()


    # style.SetTextColour(wx.BLUE)
    # style.SetFontUnderlined(True)

    def OnStrikeThrough(self, evt):
        # print self.HasSelection()
        # if self.HasSelection():
        #     r = self.GetSelectionRange()
        #     style = wx.richtext.TextAttrEx()
        #     if self.GetStyleForRange(r, style):
        #         style.SetTextEffects(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
        #         style.SetTextEffectFlags(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
        #     print style
        #     print r
        #     print
        #     self.SetStyle(r, style)

        if not self.HasSelection():
            return

        r = self.GetSelectionRange()
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
        if self.GetStyleForRange(r, attr):
            print r
            print attr.GetTextEffects()
            print
            if not attr.GetTextEffects() ^ wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH == 0:
                attr.SetFlags(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
                attr.SetTextEffects(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
                attr.SetTextEffectFlags(wx.richtext.TEXT_ATTR_EFFECT_STRIKETHROUGH)
            else:
                attr.SetFlags(wx.richtext.TEXT_ATTR_EFFECT_DOUBLE_STRIKETHROUGH)
                attr.SetTextEffects(wx.richtext.TEXT_ATTR_EFFECT_DOUBLE_STRIKETHROUGH)
                attr.SetTextEffectFlags(wx.richtext.TEXT_ATTR_EFFECT_DOUBLE_STRIKETHROUGH)
            self.SetStyle(r, attr)


    def OnUnderline(self, evt):
        self.ApplyUnderlineToSelection()

    def OnAlignLeft(self, evt):
        self.ApplyAlignmentToSelection(wx.richtext.TEXT_ALIGNMENT_LEFT)

    def OnAlignRight(self, evt):
        self.ApplyAlignmentToSelection(wx.richtext.TEXT_ALIGNMENT_RIGHT)

    def OnAlignCenter(self, evt):
        self.ApplyAlignmentToSelection(wx.richtext.TEXT_ALIGNMENT_CENTRE)

    def OnIndentMore(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_LEFT_INDENT)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            attr.SetLeftIndent(attr.GetLeftIndent() + 100)
            attr.SetFlags(wx.richtext.TEXT_ATTR_LEFT_INDENT)
            self.SetStyle(r, attr)


    def OnIndentLess(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_LEFT_INDENT)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

        if attr.GetLeftIndent() >= 100:
            attr.SetLeftIndent(attr.GetLeftIndent() - 100)
            attr.SetFlags(wx.richtext.TEXT_ATTR_LEFT_INDENT)
            self.SetStyle(r, attr)


    def OnParagraphSpacingMore(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_PARA_SPACING_AFTER)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            attr.SetParagraphSpacingAfter(attr.GetParagraphSpacingAfter() + 20);
            attr.SetFlags(wx.richtext.TEXT_ATTR_PARA_SPACING_AFTER)
            self.SetStyle(r, attr)


    def OnParagraphSpacingLess(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_PARA_SPACING_AFTER)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            if attr.GetParagraphSpacingAfter() >= 20:
                attr.SetParagraphSpacingAfter(attr.GetParagraphSpacingAfter() - 20);
                attr.SetFlags(wx.richtext.TEXT_ATTR_PARA_SPACING_AFTER)
                self.SetStyle(r, attr)


    def OnLineSpacingSingle(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
            attr.SetLineSpacing(10)
            self.SetStyle(r, attr)


    def OnLineSpacingHalf(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
            attr.SetLineSpacing(15)
            self.SetStyle(r, attr)


    def OnLineSpacingDouble(self, evt):
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
        ip = self.GetInsertionPoint()
        if self.GetStyle(ip, attr):
            r = wx.richtext.RichTextRange(ip, ip)
            if self.HasSelection():
                r = self.GetSelectionRange()

            attr.SetFlags(wx.richtext.TEXT_ATTR_LINE_SPACING)
            attr.SetLineSpacing(20)
            self.SetStyle(r, attr)


    def OnFont(self, evt):
        if not self.HasSelection():
            return

        r = self.GetSelectionRange()
        fontData = wx.FontData()
        fontData.EnableEffects(False)
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_FONT)
        if self.GetStyle(self.GetInsertionPoint(), attr):
            fontData.SetInitialFont(attr.GetFont())

        dlg = wx.FontDialog(self, fontData)
        if dlg.ShowModal() == wx.ID_OK:
            fontData = dlg.GetFontData()
            font = fontData.GetChosenFont()
            if font:
                attr.SetFlags(wx.richtext.TEXT_ATTR_FONT)
                attr.SetFont(font)
                self.SetStyle(r, attr)
        dlg.Destroy()


    def OnColour(self, evt):
        colourData = wx.ColourData()
        attr = wx.richtext.TextAttrEx()
        attr.SetFlags(wx.richtext.TEXT_ATTR_TEXT_COLOUR)
        if self.GetStyle(self.GetInsertionPoint(), attr):
            colourData.SetColour(attr.GetTextColour())

        dlg = wx.ColourDialog(self, colourData)
        if dlg.ShowModal() == wx.ID_OK:
            colourData = dlg.GetColourData()
            colour = colourData.GetColour()
            if colour:
                if not self.HasSelection():
                    self.BeginTextColour(colour)
                else:
                    r = self.GetSelectionRange()
                    attr.SetFlags(wx.richtext.TEXT_ATTR_TEXT_COLOUR)
                    attr.SetTextColour(colour)
                    self.SetStyle(r, attr)
        dlg.Destroy()



    def OnUpdateBold(self, evt):
        evt.Check(self.IsSelectionBold())

    def OnUpdateItalic(self, evt):
        evt.Check(self.IsSelectionItalics())

    def OnUpdateUnderline(self, evt):
        evt.Check(self.IsSelectionUnderlined())

    def OnUpdateAlignLeft(self, evt):
        evt.Check(self.IsSelectionAligned(wx.richtext.TEXT_ALIGNMENT_LEFT))

    def OnUpdateAlignCenter(self, evt):
        evt.Check(self.IsSelectionAligned(wx.richtext.TEXT_ALIGNMENT_CENTRE))

    def OnUpdateAlignRight(self, evt):
        evt.Check(self.IsSelectionAligned(wx.richtext.TEXT_ALIGNMENT_RIGHT))

    def OnCopy(self, evt):
        self.ProcessEvent(evt)

    def OnPaste(self, evt):
        self.ProcessEvent(evt)

    def OnRedo(self, evt):
        self.ProcessEvent(evt)

    def OnCut(self, evt):
        self.ProcessEvent(evt)

    def OnUndo(self, evt):
        self.ProcessEvent(evt)

    def ForwardEvent(self, evt):
        # The RichTextCtrl can handle menu and update events for undo,
        # redo, cut, copy, paste, delete, and select all, so just
        # forward the event to it.
        self.ProcessEvent(evt)

    ### PageSetup, Print ###
    def OnPageSetup(self, evt):
        data = wx.PageSetupDialogData()
        data.SetMarginTopLeft( (15, 15) )
        data.SetMarginBottomRight( (15, 15) )
        #data.SetDefaultMinMargins(True)
        data.SetPaperId(wx.PAPER_LETTER)

        dlg = wx.PageSetupDialog(self, data)

        if dlg.ShowModal() == wx.ID_OK:
            data = dlg.GetPageSetupData()
            tl = data.GetMarginTopLeft()
            br = data.GetMarginBottomRight()
            # self.log.WriteText('Margins are: %s %s\n' % (str(tl), str(br)))

        dlg.Destroy()


    def OnPrint(self, evt):
        data = wx.PrintDialogData()

        data.EnableSelection(True)
        data.EnablePrintToFile(True)
        data.EnablePageNumbers(True)
        data.SetMinPage(1)
        data.SetMaxPage(5)
        data.SetAllPages(True)

        dlg = wx.PrintDialog(self, data)

        if dlg.ShowModal() == wx.ID_OK:
            data = dlg.GetPrintDialogData()
            # self.log.WriteText('GetAllPages: %d\n' % data.GetAllPages())

        dlg.Destroy()



