################################################################################
# Name     : Speech.py                                                         #
# Brief    : Speech recognition module for language learning                   #
#                                                                              #
# Url      : https://www.google.com/intl/en/chrome/demos/speech.html           #
#                                                        #
#          #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import json
import pyaudio
import wave
import audioop
from collections import deque
import os
import urllib2
import time
import logging
import Queue




__default_logging_file__ = "speech.log"
logging.basicConfig(filename=__default_logging_file__, filemode='w', level=logging.DEBUG)

LOG = logging.getLogger("Speech")
LOG_GATE = True


### check whether command available ###
def which(cmd):
    import os
    def is_exe(fpath):
        return os.path.isfile(fpath) and os.access(fpath, os.X_OK)
    fpath, fname = os.path.split(cmd)
    if fpath:
        if is_exe(cmd):
            return cmd
    else:
        for path in os.environ["PATH"].split(os.pathsep):
            path = path.strip('"')
            exe_file = os.path.join(path, cmd)
            if is_exe(exe_file):
                return exe_file
    return None

###
##  view-source:https://www.google.com/intl/en/chrome/demos/speech.html
#   Supported languages and countries
#
__supported_languages__ = '''[
     ['Afrikaans', ['af-ZA']],
     ['Bahasa Indonesia',['id-ID']],
     ['Bahasa Melayu',   ['ms-MY']],
     ['Catal\xc3\xa0',    ['ca-ES']],
     ['\xc4\x8ce\xc5\xa1tina',   ['cs-CZ']],
     ['Deutsch',   ['de-DE']],
     ['English',   ['en-AU', 'Australia'],
                     ['en-CA', 'Canada'],
                     ['en-IN', 'India'],
                     ['en-NZ', 'New Zealand'],
                     ['en-ZA', 'South Africa'],
                     ['en-GB', 'United Kingdom'],
                     ['en-US', 'United States']],
     ['Espa\xc3\xb1ol',   ['es-AR', 'Argentina'],
                         ['es-BO', 'Bolivia'],
                         ['es-CL', 'Chile'],
                         ['es-CO', 'Colombia'],
                         ['es-CR', 'Costa Rica'],
                         ['es-EC', 'Ecuador'],
                         ['es-SV', 'El Salvador'],
                         ['es-ES', 'Espa\xc3\xb1a'],
                         ['es-US', 'Estados Unidos'],
                         ['es-GT', 'Guatemala'],
                         ['es-HN', 'Honduras'],
                         ['es-MX', 'M\xc3\xa9xico'],
                         ['es-NI', 'Nicaragua'],
                         ['es-PA', 'Panam\xc3\xa1'],
                         ['es-PY', 'Paraguay'],
                         ['es-PE', 'Per\xc3\xba'],
                         ['es-PR', 'Puerto Rico'],
                         ['es-DO', 'Rep\xc3\xbablica Dominicana'],
                         ['es-UY', 'Uruguay'],
                         ['es-VE', 'Venezuela']],
     ['Euskara',   ['eu-ES']],
     ['Fran\xc3\xa7ais',  ['fr-FR']],
     ['Galego',    ['gl-ES']],
     ['Hrvatski',  ['hr_HR']],
     ['IsiZulu',   ['zu-ZA']],
     ['\xc3\x8dslenska',  ['is-IS']],
     ['Italiano',  ['it-IT', 'Italia'],
                         ['it-CH', 'Svizzera']],
     ['Magyar',    ['hu-HU']],
     ['Nederlands',      ['nl-NL']],
     ['Norsk bokm\xc3\xa5l',    ['nb-NO']],
     ['Polski',    ['pl-PL']],
     ['Portugu\xc3\xaas', ['pt-BR', 'Brasil'],
                         ['pt-PT', 'Portugal']],
     ['Rom\xc3\xa2n\xc4\x83',    ['ro-RO']],
     ['Sloven\xc4\x8dina',      ['sk-SK']],
     ['Suomi',     ['fi-FI']],
     ['Svenska',   ['sv-SE']],
     ['T\xc3\xbcrk\xc3\xa7e',    ['tr-TR']],
     ['\xd0\xb1\xd1\x8a\xd0\xbb\xd0\xb3\xd0\xb0\xd1\x80\xd1\x81\xd0\xba\xd0\xb8', ['bg-BG']],
     ['P\xd1\x83\xd1\x81\xd1\x81\xd0\xba\xd0\xb8\xd0\xb9',   ['ru-RU']],
     ['\xd0\xa1\xd1\x80\xd0\xbf\xd1\x81\xd0\xba\xd0\xb8',    ['sr-RS']],
     ['\xed\x95\x9c\xea\xb5\xad\xec\x96\xb4',      ['ko-KR']],
     ['\xe4\xb8\xad\xe6\x96\x87',       ['cmn-Hans-CN', '\xe6\x99\xae\xe9\x80\x9a\xe8\xaf\x9d (\xe4\xb8\xad\xe5\x9b\xbd\xe5\xa4\xa7\xe9\x99\x86)'],
                         ['cmn-Hans-HK', '\xe6\x99\xae\xe9\x80\x9a\xe8\xaf\x9d (\xe9\xa6\x99\xe6\xb8\xaf)'],
                         ['cmn-Hant-TW', '\xe4\xb8\xad\xe6\x96\x87 (\xe5\x8f\xb0\xe7\x81\xa3)'],
                         ['yue-Hant-HK', '\xe7\xb2\xb5\xe8\xaa\x9e (\xe9\xa6\x99\xe6\xb8\xaf)']],
     ['\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e',     ['ja-JP']],
     ['Lingua lat\xc4\xabna',   ['la']]]
 '''

class Country(object):
    def __init__(self, list):
        self.Id = list[0]
        if len(list) == 2:
            self.Name = list[1]
        else:
            self.Name = None

### @End class Country

class Language(object):
    def __init__(self, list):
        self.Name = list[0]
        self.Countries = []
        for i in range(1, len(list)):
            self.Countries.append(Country(list[i]))

### @End class Language

class GoogleLanguages(object):
    LanguageList = eval(__supported_languages__.replace('\n', ''))
    __languages_map__ = {}
    @staticmethod
    def languages():
        if len(GoogleLanguages.__languages_map__) == 0:
            for language in GoogleLanguages.LanguageList:
                l = Language(language)
                GoogleLanguages.__languages_map__.append (l.Name,l)
        return GoogleLanguages.__languages_map__

### @End class GoogleLanguages


        #post it
        # lang_code='en-US'
        # googl_speech_url = 'https://www.google.com/speech-api/v1/recognize?xjerr=1&client=chromium&pfilter=2&lang=%s&maxresults=6'%(lang_code)
        # hrs = {"User-Agent": "Mozilla/5.0 (X11; Linux i686) AppleWebKit/535.7 (KHTML, like Gecko) Chrome/16.0.912.63 Safari/535.7",'Content-type': 'audio/x-flac; rate='+ str(Pygsr.rate)}
        # req = urllib2.Request(googl_speech_url, data=flac_cont, headers=hrs)



###
##    http://sebastian.germes.in/blog/2011/09/googles-speech-api/
#
#'''
#Requirements:
#
#A .FLAC file that stores your recorded speech
#The tool curl, to perform the HTTP POST request.
#Then you can do the following:
#
#curl -i -X POST -H "Content-Type:audio/x-flac; rate=12000" \
#    -T test.flac \
#    "https://www.google.com/speech-api/v1/recognize?xjerr=1\
#    &client=chromium&lang=en-EN&maxresults=10&pfilter=0"
#    Explanation of parameters:
#
#-H "Content-Type:audio/x-flac; rate=12000"
#    This tells the Google server that we send a .FLAC file with the bitrate of 12000 Hz.
#-T test.flac
#    This attaches the content of the test.flac file to the HTTP POST
#    "https://www.google.com/speech-api/v1/recognize?xjerr=1&client=chromium&lang=en-US&maxresults=10&pfilter=0"
#    The full URL to send the data to, which breaks down into:
#    https://www.google.com/speech-api/v1/recognize
#    The base address.
#xjerr=1
#    Tell speech recognition server to return errors as part of the JSON response and not HTTP error codes
#client=chromium
#    Can be anything, "chromium" is the standard one.
#lang=en-US
#    The language to be used, so far tested with en-US, en-GB, de-DE
#maxresults=10
#    The maximum results of hypotheses to be returned, default is 1.
#pfilter=0
#    This is a funny one. Google (by default) censors the results, leading to "Please search for ###" (pfilter!=0) instead of "Please search for s e x" (pfilter=0).
#
#'''


###
## The return of Google Speech Recognition API.
#  {
#    "status":0, "id":"",
#    "hypotheses":[
#       {"utterance":"haunt you","confidence":0.52503961},
#       {"utterance":"han you"},
#       {"utterance":"how are you"},
#       {"utterance":"hon you"},
#       {"utterance":"han u"},
#       {"utterance":"hon u"}
#    ] }
#
class Recognition(object):
    @staticmethod
    def load(map={}):
        res = None
        if map.has_key('utterance'):
            if map.has_key('confidence'):
                res = Recognition(map['utterance'],map['confidence'])
            else:
                res = Recognition(map['utterance'])
        return res

    def __init__(self, utterance, confidence = None):
        self.utterance = utterance
        if confidence is not None:
            self.confidence = float(confidence)
        else:
            self.confidence = 0

    def __str__(self):
        res = r'{ "utterance":' + self.utterance
        if self.confidence > 0:
            res = res + r'" ,"confidence": "' + str(self.confidence)
        res = res + r' } '
        return res

### @End class Recognition

class HTTPRequest(object):
    import socket
    timeout = socket._GLOBAL_DEFAULT_TIMEOUT,
    def __init__(self, lang, url, useragent, mediatype, gzip, timeout,
                 *args, **kwargs):
        self.lang = lang
        if self.lang.lower() not in __supported_languages__.lower():
            err= "Unknown lang=%s"%(self.lang)
            raise Exception(err)
        self.url = url.lower()
        if r'?' not in self.url:
            err = "Wrong format url=(%s), should be similar to " \
                  "(https://www.google.com/speech-api/v1/recognize?xjerr=1&client=chromium&pfilter=2)" %(self.url)
            raise Exception(err)
        self.gzip = gzip
        self.useragent = useragent
        self.mediatype = mediatype
        self.timeout = timeout
        self.contenttype = self.mediatype + ';'
        pass

    def getContentType(self):
        return self.contenttype

    def getHeader(self):
        ### http://en.wikipedia.org/wiki/List_of_HTTP_header_fields ###
        self.headers = {
            'User-Agent': self.useragent,
            ### gstatic is another company owned by google ###
            'Referer': r'http://www.gstatic.com/s2/sitemaps/profiles-sitemap.xml',
            'Content-Type':self.getContentType(),
        }

        if self.gzip :
            self.headers['Accept-Encoding'] = 'gzip, deflate'
        return self.headers


    def urlAppend(self, key, val):
        import re
        ### http://docs.python.org/2/library/re.html#re.sub ###
        ### replace original key ###
        self.url = re.sub(str(key).strip()+r'=[^&]*&?', '', self.url)
        ### replace last '&', if there is ####
        self.url = re.sub(r'&$', '', self.url)
        self.url = str(self.url).strip() + '&'+ str(key).strip() + '=' + str(val).strip()

    def requestByFile(self, localFile):
        fp = open(localFile, "rb")
        try:
            data = fp.read()
        finally:
            fp.close()
        if data:
            return self.request(data)
        else:
            err = "Fail to read file=%s"%(localFile)
            raise Exception(err)

    def request(self, data):
        ### use urllib2 for request ###
        ### http://docs.python.org/2/howto/urllib2.html ###
        req = urllib2.Request(self.url, headers=self.getHeader(), data=self.preprocess(data))
        if LOG_GATE:
            LOG.debug("HTTPRequest.request(): url=%s, header=%s, type(data)=%s",
                      self.url, str(self.headers), type(data))
        ### In case of proxy requirement ###
        ##
        #    os.environ['http_proxy'] = '$hostname:$port'
        #    os.environ['no_proxy'] = '$hostname'
        #
        urllib2.install_opener(
            urllib2.build_opener(
                urllib2.ProxyHandler()
            )
        )
        ### open after request ###
        urlop = urllib2.urlopen(req)
        if LOG_GATE:
            LOG.debug("HTTPRequest.request(): timeout=%s", str(self.timeout))
        ### read the opened socket ###
        red = urlop.read()
        if LOG_GATE:
            LOG.debug("HTTPRequest.request(): content-encoding=%s", str(urlop.headers.get('content-encoding', '')))
        ### unzip compressed data ###
        self.gzip = ('gzip' in urlop.headers.get('content-encoding', ''))
        if self.gzip:
            import zlib
            red = zlib.decompress(red, 16+zlib.MAX_WBITS)
        ### interpret data  ###
        return self.interpret(red)

    def preprocess(self, data):
        return data

    def interpret(self, data):
        return data



###
##  http://www.hung-truong.com/blog/2013/04/26/hacking-googles-text-to-speech-api/
#
#
#   http://translate.google.com/translate_tts?ie=UTF-8&q=hello%20world&tl=en&total=1&idx=0&textlen=11&prev=input
#
#
#  Breaking down the parameters,
# "ie" is the text's encoding,
# "q" is the text to convert to audio,
# "tl" is the text language,
# "total" is the total number of chunks (more on that later),
# "idx" is which chunk we're on,
# "textlen" is the length of the text in that chunk and
# "prev" is not really important.
#
#


class GoogleTTSRequest(HTTPRequest):
    def __init__(self, lang='en-US',
                url= r'http://translate.google.com/translate_tts?tl=en&q=',
                useragent = r'Mozilla/5.0 (X11; Linux i686) AppleWebKit/535.7 (KHTML, like Gecko) Chrome/16.0.912.63 Safari/535.7',
                mediatype = r'audio/x-flac',
                gzip = True,
                timeout = HTTPRequest.timeout,
                encoding = 'UTF-8',
                mp3file = 'Speech.mp3'
    ):
        HTTPRequest.__init__(self, lang, url, useragent, mediatype, gzip, timeout)
        self.encoding = encoding
        self.urlAppend('tl', self.lang)
        self.urlAppend('ie', self.encoding)
        self.mp3file = mp3file
        pass

    def request(self, text):
        import re
        text_list = re.split('(,|:|;|\?|\.|\n|' +
                             '\xe3\x80\x82|\xef\xbc\x9f|\xef\xbc\x81|\xe3\x80\x82|\xef\xbc\x8c|\xe3\x80\x81|\xef\xbc\x9b|\xef\xbc\x9a)',
                             text)
        sentences = []
        for idx, val in enumerate(text_list):
            if idx % 2 == 1 or len(val.strip()) == 0:
                continue
            sentences.append(val.strip())
        pieces = []
        for idx, txt in enumerate(sentences) :
            print txt.decode('utf8')
            self.urlAppend('q', urllib2.quote(txt))
            self.urlAppend('total', len(sentences))
            self.urlAppend('idx', idx)
            red = HTTPRequest.request(self,"")
            pieces.append(red)
        fp = open(self.mp3file, 'wb')
        try:
            for p in pieces:
                fp.write(p)
        finally:
            fp.close()
        return pieces

    # def interpret(self, data):
    #     return data
### @End class GoogleTTSRequest


class GoogleSpeechRequest(HTTPRequest):
    def __init__(self, lang='en-US',
                url= r'https://www.google.com/speech-api/v1/recognize?xjerr=1&client=chromium&pfilter=2',
                useragent = r'Mozilla/5.0 (X11; Linux i686) AppleWebKit/535.7 (KHTML, like Gecko) Chrome/16.0.912.63 Safari/535.7',
                mediatype = r'audio/x-flac',
                gzip = True,
                timeout = HTTPRequest.timeout,
                rate = 44100,
                maxresults = 4
    ):
        HTTPRequest.__init__(self, lang, url, useragent, mediatype, gzip, timeout)
        self.rate = rate
        self.maxresults = maxresults
        self.urlAppend('lang', self.lang)
        self.urlAppend('maxresults', self.maxresults)
        ### s_e_x is not allowed ###
        if "pfilter=0" in self.url:
            self.url = str(self.url).replace("pfilter=0", "pfilter=2")
        elif 'pfilter' not in self.url:
            self.urlAppend('pfilter',2)
        self.urlAppend('xjerr',1)
        self.urlAppend('client','chromium')
        if LOG_GATE:
            LOG.debug("GoogleSpeechRequest.__init__(): url=%s", self.url)

    def getContentType(self):
        self.contenttype = self.mediatype + '; rate=' + str(self.rate)
        return self.contenttype

    def interpret(self, red):
        ### split into lines ##
        josnres = []
        lines = str(red).split('\n')
        for lin in lines:
            if LOG_GATE:
                LOG.debug("GoogleSpeechRequest.request(): line=%s", lin)
            if '{' in lin and '}' in lin and 'utterance' in lin:
                if LOG_GATE:
                        LOG.debug("GoogleSpeechRequest.request(): lin=%s", lin)
                josnres.append(json.loads(lin))
        rets = []
        for res in josnres:
            for utt in res['hypotheses']:
                rec = Recognition.load(utt)
                if rec is not None:
                    rets.append(rec)
                    if LOG_GATE:
                        LOG.debug("GoogleSpeechRequest.request(): recognized=%s", str(rec))

        return rets

### @End class GoogleSpeechRequest

class PyAudioRecord(object):

    def __init__(self, wavefile = 'Speech.wav', format=pyaudio.paInt16, channels=1, rate=44100, frames_per_buffer=1024):
        self.wavefile = wavefile
        self.format = format
        self.channels = channels
        self.rate = rate
        self.frames_per_buffer = frames_per_buffer
        self.audio = pyaudio.PyAudio()
        self.stop = False
        self.frames = Queue.Queue()

    def record(self, time = -1):
        stream = self.audio.open(format=self.format, channels=self.channels,
                            rate=self.rate, input=True,
                            frames_per_buffer=self.frames_per_buffer)
        print "... Record Start:"
        try:
            if time < 0:
                while not self.stop:
                    data = stream.read(self.frames_per_buffer)
                    self.frames.put(data)
            else:
                for i in range(0, time * self.rate / self.frames_per_buffer + 1):
                    data = stream.read(self.frames_per_buffer)
                    self.frames.put(data)
                self.stop = True
        finally:
            stream.stop_stream()
            stream.close()
        self.audio.terminate()
        print "Record Stop ..."

    def recordToFile(self, time = -1):
        import thread
        thread.start_new_thread(self.record, (time, ))
        writefp = wave.open(self.wavefile, 'wb')
        writefp.setnchannels(self.channels)
        writefp.setsampwidth(self.audio.get_sample_size(self.format))
        writefp.setframerate(self.rate)
        try:
            while not self.stop:
                writefp.writeframes(''.join(self.frames.get()))
        finally:
            writefp.close()
        ### print wave file  ###
        print "See ", self.wavefile


    ### define several commands ###
    CMD_CONV = {
         # We need a WAV to FLAC converter.
        'flac' : ('-f'),
        # system("sox %s -t wav -r 8000 -t flac %s.flac" % (self.file, self.file))
    }

    def convertToFlac(self, wavefile = None):
        ### http://stackoverflow.com/questions/377017/test-if-executable-exists-in-python  ###
        ### first define function to check whether command available. ###
        if wavefile is None:
            wavefile = self.wavefile
        flacfile = wavefile.replace('.wav', '.flac')

        def flac(wavefile):
            ### check which function available for convert ###
            for cmd in PyAudioRecord.CMD_CONV.keys():
                if which(cmd):
                    params = [cmd, PyAudioRecord.CMD_CONV[cmd], wavefile]
                    import subprocess
                    subprocess.call(params)
                    if LOG_GATE:
                        LOG.debug("PyAudioRecord.convertToFlac(): cmd=%s", str(' '.join(params)))
                    break
            pass


        # sox(wavefile)

        from Portable import Portable
        # print os.getenv('PATH')
        Portable.call('flac', '-f', wavefile)
        if os.path.exists(flacfile):
            return flacfile
        else:
            err = "Fail to convert file %s"%(wavefile)
            raise Exception(err)

def runSR():
    rcd = PyAudioRecord()
    rcd.recordToFile(4)
    flacFile =  rcd.convertToFlac()
    grq = GoogleSpeechRequest()
    res = grq.requestByFile(flacFile)
    print str(res)
    for x in res:
        print str(x)

def runTTS(txtf):
    gtr = GoogleTTSRequest('cmn-Hans-CN')
    gtr.requestByFile(txtf)

#
if __name__ == '__main__':
    import sys
    # runTTS(sys.argv[1])
    runSR()