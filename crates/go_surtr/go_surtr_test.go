package go_surtr

import "testing"

func TestGenerateSurtFromUrl(t *testing.T) {
	testCases := []struct {
		url         string
		expected    string
		shouldError bool
	}{
		{
			url:         "",
			expected:    "",
			shouldError: true,
		},
		{
			url:         "filedesc:foo.arc.gz",
			expected:    "filedesc:foo.arc.gz",
			shouldError: false,
		},
		{
			url:         "filedesc:/foo.arc.gz",
			expected:    "filedesc:/foo.arc.gz",
			shouldError: false,
		},
		{
			url:         "filedesc://foo.arc.gz",
			expected:    "filedesc://foo.arc.gz",
			shouldError: false,
		},
		{
			url:         "warcinfo:foo.warc.gz",
			expected:    "warcinfo:foo.warc.gz",
			shouldError: false,
		},
		{
			url:         "dns:alexa.com",
			expected:    "dns:alexa.com",
			shouldError: false,
		},
		{
			url:         "dns:archive.org",
			expected:    "dns:archive.org",
			shouldError: false,
		},
		{
			url:         "http://www.archive.org/",
			expected:    "org,archive)/",
			shouldError: false,
		},
		{
			url:         "http://archive.org/",
			expected:    "org,archive)/",
			shouldError: false,
		},
		{
			url:         "http://archive.org/goo/",
			expected:    "org,archive)/goo",
			shouldError: false,
		},
		{
			url:         "http://archive.org/goo/?",
			expected:    "org,archive)/goo",
			shouldError: false,
		},
		{
			url:         "http://archive.org/goo/?b&a",
			expected:    "org,archive)/goo?a&b",
			shouldError: false,
		},
		{
			url:         "http://archive.org/goo/?a=2&b&a=1",
			expected:    "org,archive)/goo?a=1&a=2&b",
			shouldError: false,
		},
		{
			url:         "http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221",
			expected:    "org,archive)/index.php?action=profile;u=4221",
			shouldError: false,
		},
		// PHP Session ID
		{
			url:         "http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221",
			expected:    "org,archive)/index.php?action=profile;u=4221",
			shouldError: false,
		},
		// WHOIS url
		{
			url:         "whois://whois.isoc.org.il/shaveh.co.il",
			expected:    "il,org,isoc,whois)/shaveh.co.il",
			shouldError: false,
		},
		// Yahoo web bug. See https://github.com/internetarchive/surt/issues/1
		{
			url:         "http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2",
			expected:    "com,yahoo,webhosting,visit)/visit.gif?&b=netscape%205.0%20(windows;%20en-us)&c=24&j=true&o=win32&r=http://web.archive.org/web/20090517140029/http://anthonystewarthead.electric-chi.com/&s=1366x768&v=1.2",
			shouldError: false,
		},
		{
			url:         "http://example.com/app?item=Wroc%C5%82aw",
			expected:    "com,example)/app?item=wroc%c5%82aw",
			shouldError: false,
		},
	}

	for _, testCase := range testCases {
		result, err := GenerateSurtFromURL(testCase.url)
		if err != nil && !testCase.shouldError {
			t.Errorf("Expected no error for URL: %s, but got: %v", testCase.url, err)
		}
		if err == nil && testCase.shouldError {
			t.Errorf("Expected error for URL: %s, but got no error", testCase.url)
		}
		if result != testCase.expected {
			t.Errorf("Expected result: %s, but got: %s", testCase.expected, result)
		}
	}
}

func TestGenerateSurtFromURLOptions(t *testing.T) {
	testCases := []struct {
		url         string
		expected    string
		options     map[string]bool
		shouldError bool
	}{
		{
			url:      "http://archive.org/goo/?a=2&b&a=1",
			expected: "org,archive,)/goo?a=1&a=2&b",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "dns:archive.org",
			expected: "dns:archive.org",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "warcinfo:foo.warc.gz",
			expected: "warcinfo:foo.warc.gz",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},

		// Simple customization tests
		{
			url:      "mailto:foo@example.com",
			expected: "mailto:foo@example.com",
			options:  map[string]bool{},
		},
		{
			url:      "http://www.example.com/",
			expected: "http://(com,example)/",
			options: map[string]bool{
				"with_scheme": true,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "http://(com,example)/",
			options: map[string]bool{
				"with_scheme":  true,
				"host_massage": true,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "com,example)/",
			options: map[string]bool{
				"with_scheme": false,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "http://(com,example,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "https://www.example.com/",
			expected: "https://(com,example,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "ftp://www.example.com/",
			expected: "com,example,)/",
			options: map[string]bool{
				"with_scheme":    false,
				"trailing_comma": true,
			},
		},
		{
			url:      "ftp://www.example.com/",
			expected: "com,example)/",
			options: map[string]bool{
				"with_scheme":    false,
				"trailing_comma": false,
			},
		},
		{
			url:      "ftp://www.example.com/",
			expected: "ftp://(com,example,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "http://(com,example,www)/",
			options: map[string]bool{
				"with_scheme":  true,
				"host_massage": false,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "com,example,www)/",
			options: map[string]bool{
				"with_scheme":  false,
				"host_massage": false,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "http://(com,example,www,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
				"host_massage":   false,
			},
		},
		{
			url:      "https://www.example.com/",
			expected: "https://(com,example,www,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
				"host_massage":   false,
			},
		},
		{
			url:      "ftp://www.example.com/",
			expected: "ftp://(com,example,www,)/",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
				"host_massage":   false,
			},
		},

		// Special protocol tests
		{
			url:      "mailto:foo@example.com",
			expected: "mailto:foo@example.com",
			options: map[string]bool{
				"with_scheme": true,
			},
		},
		{
			url:      "mailto:foo@example.com",
			expected: "mailto:foo@example.com",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "mailto:foo@example.com",
			expected: "mailto:foo@example.com",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "dns:archive.org",
			expected: "dns:archive.org",
			options: map[string]bool{
				"with_scheme": true,
			},
		},
		{
			url:      "dns:archive.org",
			expected: "dns:archive.org",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "dns:archive.org",
			expected: "dns:archive.org",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "whois://whois.isoc.org.il/shaveh.co.il",
			expected: "whois://(il,org,isoc,whois)/shaveh.co.il",
			options: map[string]bool{
				"with_scheme": true,
			},
		},
		{
			url:      "whois://whois.isoc.org.il/shaveh.co.il",
			expected: "il,org,isoc,whois,)/shaveh.co.il",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "whois://whois.isoc.org.il/shaveh.co.il",
			expected: "whois://(il,org,isoc,whois,)/shaveh.co.il",
			options: map[string]bool{
				"trailing_comma": true,
				"with_scheme":    true,
			},
		},
		{
			url:      "warcinfo:foo.warc.gz",
			expected: "warcinfo:foo.warc.gz",
			options: map[string]bool{
				"trailing_comma": true,
			},
		},
		{
			url:      "warcinfo:foo.warc.gz",
			expected: "warcinfo:foo.warc.gz",
			options: map[string]bool{
				"with_scheme": true,
			},
		},
		{
			url:      "warcinfo:foo.warc.gz",
			expected: "warcinfo:foo.warc.gz",
			options: map[string]bool{
				"with_scheme":    true,
				"trailing_comma": true,
			},
		},
		{
			url:      "http://www.example.com/",
			expected: "com,example)/",
			options: map[string]bool{
				"reverse_ipaddr": false,
			},
		},
		{
			url:      "http://192.168.1.254/info/",
			expected: "254,1,168,192)/info",
			options:  map[string]bool{},
		},
		{
			url:      "http://192.168.1.254/info/",
			expected: "254,1,168,192)/info",
			options: map[string]bool{
				"reverse_ipaddr": true,
			},
		},
		{
			url:      "http://192.168.1.254/info/",
			expected: "192.168.1.254)/info",
			options: map[string]bool{
				"reverse_ipaddr": false,
			},
		},
	}

	for _, testCase := range testCases {
		result, err := generateSurtFromURLOptions(testCase.url, testCase.options)
		if err != nil && !testCase.shouldError {
			t.Errorf("Expected no error for URL: %s, but got: %v", testCase.url, err)
		}
		if err == nil && testCase.shouldError {
			t.Errorf("Expected error for URL: %s, but got no error", testCase.url)
		}
		if result != testCase.expected {
			t.Errorf("Expected result: %s, but got: %s", testCase.expected, result)
		}
	}
}
