#!/usr/bin/env python



from ...bibiparrot.Constants.constants import __required_wx_version__

import unittest, os, sys
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.tools

from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.Configurations.Configuration import log

from ...thirdparty.pyth.pyth.plugins.xhtml.reader import XHTMLReader
from ...thirdparty.pyth.pyth.plugins.xhtml.writer import XHTMLWriter

from ...thirdparty.pyth.pyth.plugins.rtf15.reader import Rtf15Reader
from ...thirdparty.pyth.pyth.plugins.rtf15.writer import Rtf15Writer

from cStringIO import StringIO

class TestUIElement(unittest.TestCase):
    def setUp(self):

        self.content = StringIO(unicode(r"""
          <div>
            <p><strong>Simple document</strong></p>
            <p><i>this document has

            </i>no hypertext links yet.</p>
            <p><strong>bold text.</strong> <em>italic text.</em></p>
            <p class=important>bold text from css style
              <em> this is bold and italic</em>
            </p>
            <p class=bold> this is bold too</p>
            <p>unicode characters : hello</p>
            <p style="font-weight: bold">bold too</p>
            <p>
              example<span style="vertical-align: super"> super </span>
              example<span style="vertical-align: sub"> sub </span>
            </p>
            a list
            <ul>
              <li>hello
              test</li>
              <li>bonjour</li>
              <li>guten tag</li>
            </ul>
            <p>
              <a href=http://www.google.com>a link
              </a> single space here.
              <br/>a br tag
            </p>
          </div>
        """))

        self.css = """
          .important {font-weight: bold}
          p.bold {font-weight: bold}
          .other {font-weight: normal; color: blue}
        """

        self.html_doc = '''
                    <p></p>
            <ol type="1"><li> <font face="Lucida Grande" size="3" color="#000000" >Numbered bullets are possible, again using sub-indents:</font>
            <li> <font face="Lucida Grande" size="3" color="#000000" >This is my first item. Note that wxRichTextCtrl doesn't automatically do numbering, but this will be added later.</font>
            </ol><p align="left"><font face="Lucida Grande" size="3" color="#000000" >This is my second item.</font></p>
        '''

        pass

    def tearDown(self):
        pass

    # def testPyth(self):
    #     doc = XHTMLReader.read(self.content, self.css)
    #     print XHTMLWriter.write(doc).getvalue()
    #     pass

    # def testWxHTML(self):
    #     from bs4 import BeautifulSoup
    #     fp = open('/Users/shi/Desktop/hello.html', 'r')
    #     html_doc = fp.read()
    #     fp.close()
    #
    #
    #     # soup = BeautifulSoup(html_doc)
    #     # print soup.body
    #     doc = XHTMLReader.read(html_doc,"")
    #     print unicode(XHTMLWriter.write(doc).getvalue())
    #     pass

    def testWxRTF(self):
        # from bs4 import BeautifulSoup
        # fp = open('/Users/shi/Desktop/hello.html', 'r')
        # html_doc = fp.read()
        # fp.close()

        # soup = BeautifulSoup(html_doc)
        # print soup.body
        # doc = XHTMLReader.read(html_doc,"")
        # fp = open('/Users/shi/Desktop/hello.rtf', 'wb')
        # fp.write(Rtf15Writer.write(doc).getvalue())
        # print unicode(XHTMLWriter.write(doc).getvalue())
        # html_doc = fp.read()
        # fp.close()
        # print html_doc
        pass


if __name__ == '__main__':
    unittest.main()