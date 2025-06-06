import pytest
import py_surtr as surt


def test_surt():
    # These tests are from WaybackURLKeyMakerTest.java
    assert surt.surt(None) == "-"
    assert surt.surt("") == "-"
    assert surt.surt("filedesc:foo.arc.gz") == "filedesc:foo.arc.gz"
    assert surt.surt("filedesc:/foo.arc.gz") == "filedesc:/foo.arc.gz"
    assert surt.surt("filedesc://foo.arc.gz") == "filedesc://foo.arc.gz"
    assert surt.surt("warcinfo:foo.warc.gz") == "warcinfo:foo.warc.gz"
    assert surt.surt("dns:alexa.com") == "dns:alexa.com"
    assert surt.surt("dns:archive.org") == "dns:archive.org"

    assert surt.surt("http://www.archive.org/") == "org,archive)/"
    assert surt.surt("http://archive.org/") == "org,archive)/"
    assert surt.surt("http://archive.org/goo/") == "org,archive)/goo"
    assert surt.surt("http://archive.org/goo/?") == "org,archive)/goo"
    assert surt.surt("http://archive.org/goo/?b&a") == "org,archive)/goo?a&b"
    assert (
        surt.surt("http://archive.org/goo/?a=2&b&a=1")
        == "org,archive)/goo?a=1&a=2&b"
    )

    # trailing comma mode
    assert (
        surt.surt("http://archive.org/goo/?a=2&b&a=1", trailing_comma=True)
        == "org,archive,)/goo?a=1&a=2&b"
    )
    assert (
        surt.surt("dns:archive.org", trailing_comma=True) == "dns:archive.org"
    )
    assert (
        surt.surt("warcinfo:foo.warc.gz", trailing_comma=True)
        == "warcinfo:foo.warc.gz"
    )

    # PHP session id:
    assert (
        surt.surt(
            "http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221"
        )
        == "org,archive)/index.php?action=profile;u=4221"
    )

    # WHOIS url:
    assert (
        surt.surt("whois://whois.isoc.org.il/shaveh.co.il")
        == "il,org,isoc,whois)/shaveh.co.il"
    )

    # Yahoo web bug. See https://github.com/internetarchive/surt/issues/1
    assert (
        surt.surt(
            "http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2"
        )
        == "com,yahoo,webhosting,visit)/visit.gif?&b=netscape%205.0%20(windows;%20en-us)&c=24&j=true&o=win32&r=http://web.archive.org/web/20090517140029/http://anthonystewarthead.electric-chi.com/&s=1366x768&v=1.2"
    )

    # Simple customization:
    # assert surt.surt("http://www.example.com/", canonicalizer=lambda x, **opts: x) == 'com,example,www)/'
    assert surt.surt("mailto:foo@example.com") == "mailto:foo@example.com"
    assert (
        surt.surt("http://www.example.com/", with_scheme=True)
        == "http://(com,example)/"
    )
    assert (
        surt.surt(
            "http://www.example.com/", with_scheme=True, host_massage=True
        )
        == "http://(com,example)/"
    )
    assert (
        surt.surt("http://www.example.com/", with_scheme=False)
        == "com,example)/"
    )
    assert (
        surt.surt(
            "http://www.example.com/", with_scheme=True, trailing_comma=True
        )
        == "http://(com,example,)/"
    )
    assert (
        surt.surt(
            "https://www.example.com/", with_scheme=True, trailing_comma=True
        )
        == "https://(com,example,)/"
    )
    assert (
        surt.surt(
            "ftp://www.example.com/", with_scheme=False, trailing_comma=True
        )
        == "com,example,)/"
    )
    assert (
        surt.surt(
            "ftp://www.example.com/", with_scheme=False, trailing_comma=False
        )
        == "com,example)/"
    )
    assert (
        surt.surt(
            "ftp://www.example.com/", with_scheme=True, trailing_comma=True
        )
        == "ftp://(com,example,)/"
    )
    assert (
        surt.surt(
            "http://www.example.com/", with_scheme=True, host_massage=False
        )
        == "http://(com,example,www)/"
    )
    assert (
        surt.surt(
            "http://www.example.com/", with_scheme=False, host_massage=False
        )
        == "com,example,www)/"
    )
    assert (
        surt.surt(
            "http://www.example.com/",
            with_scheme=True,
            trailing_comma=True,
            host_massage=False,
        )
        == "http://(com,example,www,)/"
    )
    assert (
        surt.surt(
            "https://www.example.com/",
            with_scheme=True,
            trailing_comma=True,
            host_massage=False,
        )
        == "https://(com,example,www,)/"
    )
    assert (
        surt.surt(
            "ftp://www.example.com/",
            with_scheme=True,
            trailing_comma=True,
            host_massage=False,
        )
        == "ftp://(com,example,www,)/"
    )

    assert (
        surt.surt("mailto:foo@example.com", with_scheme=True)
        == "mailto:foo@example.com"
    )
    assert (
        surt.surt("mailto:foo@example.com", trailing_comma=True)
        == "mailto:foo@example.com"
    )
    assert (
        surt.surt(
            "mailto:foo@example.com", with_scheme=True, trailing_comma=True
        )
        == "mailto:foo@example.com"
    )
    assert surt.surt("dns:archive.org", with_scheme=True) == "dns:archive.org"
    assert (
        surt.surt("dns:archive.org", trailing_comma=True) == "dns:archive.org"
    )
    assert (
        surt.surt("dns:archive.org", with_scheme=True, trailing_comma=True)
        == "dns:archive.org"
    )
    assert (
        surt.surt("whois://whois.isoc.org.il/shaveh.co.il", with_scheme=True)
        == "whois://(il,org,isoc,whois)/shaveh.co.il"
    )
    assert (
        surt.surt(
            "whois://whois.isoc.org.il/shaveh.co.il", trailing_comma=True
        )
        == "il,org,isoc,whois,)/shaveh.co.il"
    )
    assert (
        surt.surt(
            "whois://whois.isoc.org.il/shaveh.co.il",
            trailing_comma=True,
            with_scheme=True,
        )
        == "whois://(il,org,isoc,whois,)/shaveh.co.il"
    )
    assert (
        surt.surt("warcinfo:foo.warc.gz", trailing_comma=True)
        == "warcinfo:foo.warc.gz"
    )
    assert (
        surt.surt("warcinfo:foo.warc.gz", with_scheme=True)
        == "warcinfo:foo.warc.gz"
    )
    assert (
        surt.surt(
            "warcinfo:foo.warc.gz", with_scheme=True, trailing_comma=True
        )
        == "warcinfo:foo.warc.gz"
    )


@pytest.mark.xfail(reason="a bug not yet fixed for compatibility concern")
def test_surt_query():
    assert (
        surt.surt("http://example.com/script?type=a+b+%26+c&grape=wine")
        == "com,example)/script?grape=wine&type=a+b+%26+c"
    )


@pytest.mark.parametrize(
    "url,out",
    [
        (
            "http://example.com/app?item=Wroc%C5%82aw",
            "com,example)/app?item=wroc%c5%82aw",
        )
    ],
)
def test_surt_nonascii(url, out):
    """non-ASCII %-encoded in unicode string input"""
    assert surt.surt(url) == out


@pytest.mark.parametrize(
    "url,opts,out",
    [
        (
            "http://www.example.com/",
            dict(reverse_ipaddr=False),
            "com,example)/",
        ),
        ("http://192.168.1.254/info/", {}, "254,1,168,192)/info"),
        (
            "http://192.168.1.254/info/",
            dict(reverse_ipaddr=True),
            "254,1,168,192)/info",
        ),
        (
            "http://192.168.1.254/info/",
            dict(reverse_ipaddr=False),
            "192.168.1.254)/info",
        ),
    ],
)
def test_surt_ipaddress(url, opts, out):
    assert surt.surt(url, **opts) == out


@pytest.mark.parametrize("burl", [b"http://example.com/"])
def test_surt_return_type(burl):
    """surt.surt() returns the same type of string object (i.e. returns unicode
    string for unicode string input, and byets for bytes)

    Note this behavior may change in the future versions. This test is for
    testing compatibility until that happens.
    """
    assert isinstance(burl, bytes)

    b = surt.surt(burl)
    assert type(b) is type(burl)

    uurl = burl.decode("ascii")
    u = surt.surt(uurl)
    assert type(u) is type(uurl)
