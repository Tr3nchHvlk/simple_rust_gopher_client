<!-- <style>
    * { font-family: 'Trebuchet MS', sans-serif; }
    body {
        width: 80%;
        margin-left: 10%;
    }
    p { font-size: 1.3rem; }
    h1 { font-size: 3.3rem; }
    h2 { font-size: 2.3rem; }
    h3 { font-size: 1.9rem; }
    table p { font-size: 1.0rem; }
    p.img-caption {
        font-size: 0.9rem; 
        margin-right: 10%;
    }
    table {
        table-layout: fixed;
        width: 80%;
        margin-left: 10%;
        background-color: #f7f7f7;
        border: 1.5px solid #666;
    }
    th {
        border: 1.5px solid #666;
        border-bottom: 1.5px solid #666;
    }
    td {
        border-top: 1.5px solid #BBB;
        border-left: 1.5px solid #666;
        border-right: 1.5px solid #666;
    }
    table, th, td { 
        border-collapse: collapse;
        padding: 5px;
        color: #222;
    }
    code {
        border: 1.5px solid #DDD;
        border-radius: 4px;
        padding: 2px;
        padding-left: 5px;
        padding-right: 5px;
        background-color: #f7f7f7;
        color: #444;
    }
    pre {
        width: 95%;
        border: 1.5px solid #DDD;
        border-radius: 4px;
        padding: 17.5px;
        padding-left: 20px;
        padding-right: 20px;
        background-color: #f7f7f7;
        color: #444;
        word-wrap: break-word;
        white-space: pre-wrap;
        word-break: break-all;
        line-height: 175%;
    }
    pre code { 
        border: 1.5px solid transparent;
        padding: 2px;
        padding-left: 5px;
        padding-right: 5px;
        background-color: transparent;
        color: inherit; 
    }
    input[type=text] {
        width: 20%;
        border: 1.5px solid #666;
        border-radius: 4px;
        padding: 2px;
        padding-left: 5px;
        padding-right: 5px;
        background-color: #444;
        color: #444;
    }
    a, u {
        text-decoration-line: underline;
        text-decoration-style: dotted;
    }
    input[type=text]:hover {
        color: #f7f7f7;
    }
    input[type=checkbox] {
        pointer-events: none;
    }
    iframe {
        width: 97%;
        height: 45vw;
    }
    img {
        width: 45%;
        display: block;
        margin-left: auto;
        margin-right: auto;
    }
</style> -->

<body>

## Usage guide
---
- <p>Run program with command line argument <code>-h</code> or <code>--help</code> to see options.</p>

<br>

## gopher Client
---
<p>The purpose of a gopher Client is to ease the process of traversing through a gopher server. To initiate a gopher Client, provide the domain/ip of the target gopher server (port is defaulted to 70, can be reconfigured). Gopher Client provided functions including <i>scan_all()</i> and <i>download_all_to()</i> for scanning entire server's directory and downloading all accessible files.</p>

<br>

- <dl><dt><p><strong>gopher::io::Client</strong></p></dt>
    <dd>
    <dl>
    <dt>
    <p>new(domain) &rarr; <i>Result&lt;Client&gt;</i></p>
    </dt>
    <dd>
    <p>domain: <i>String</i>, Public domain like <i>google.com.au</i> 
    <br>or public ip address like <i>142.250.70.227</i></p>
    </dd>

    <br>

    <dt>
    <p>update_port(<i>&mut self</i>, port) &rarr; <i>&mut Client</i></p>
    </dt>
    <dd>
    <p>port: <i>u16</i></p>
    </dd>

    <br>

    <dt><p>ping(<i>&mut self</i>) &rarr; <i>Result&lt;&mut Client&gt;</i></p></dt>
    <blockquote><p>Send an initial request to ping the server</p></blockquote>
    <dd>
    </dd>

    <br>

    <dt><p>scan_all(<i>&mut self</i>) &rarr; <i>&mut Client</i></p></dt>
    <blockquote><p>Scan server directory and all sub-directories for <i>Item</i>-s</p></blockquote>
    <dd>
    </dd>

    <br>

    <dt><p>get_info_at(<i>&self</i>, referer) &rarr; <i>String</i></p></dt>
    <blockquote><p>Return a combined info text by combining text contents from all info items at a directory level specified by referer</p></blockquote>
    <dd>
    <p>referer: <i>Referer</i>, Object containing information for directory level (specified in field <i>path</i>)</p>
    </dd>

    <br>

    <dt><p>download_all_to(<i>&mut self</i>, path_prefix) &rarr; <i>Result&lt;&mut Client&gt;</i></p></dt>
    <blockquote><p>Download all items (updated by scan_all) to folder specified by path_prefix</p></blockquote>
    <dd>
    <p>path_prefix: <i>&str</i>, Folder path for which all the file will be downloaded to</p>
    </dd>

    </dl>
    </dd>
</dl>

<br>

## gopher Request/Response process
---
<p>
Initiate a <i>Request</i> instance to start the process, configure the path variable if necessary (defaulted to "/") before proceed with <i>.send()</i>. A <i>ResponseBuilder</i> instance will be returned on successful response, and then can be used to generate an array of <i>Item</i> with each item containing the information of each line parsed as a gopher item (i.e. information items or directory items). 
</p>

<br>

<blockquote>
<p>
<i>gopher::io::Request</i> &rarr; <i>gopher::io::ResponseBuilder</i>
</p>
</blockquote>

<br>

- <dl><dt><p><strong>gopher::io::Request</strong></p></dt>
    <dd>
    <dl>
    <dt>
    <p>new(domain, port) &rarr; <i>Result&lt;Request&gt;</i></p>
    </dt>
    <dd>
    <p>domain: <i>String</i>, Public domain like <i>google.com.au</i> 
    <br>or public ip address like <i>142.250.70.227</i></p>
    <p>port: <i>u16</i>, The port number for domain with a range from 0-65535</p>
    </dd>
    
    <br>

    <dt>
    <p>from_item(item) &rarr; <i>Result&lt;Request&gt;</i></p>
    </dt>
    <blockquote>
    <p>Spawn a <i>Request</i> instance from the destination of the item (that is the server which is item is from and its path on that server)</p>
    </blockquote>
    <dd>
    <p>item: <i>&Item</i></p>
    </dd>

    <br>

    <dt>
    <p>update_path(<i>&mut self</i>, new_path) &rarr; <i>&mut Request</i></p>
    </dt>
    <dd>
    <p>new_path: <i>&str</i></p>
    </dd>

    <br>

    <dt>
    <p>send(<i>&self</i>) &rarr; <i>Result&lt;ResponseBuilder&gt;</i></p>
    </dt>
    <blockquote>
    <p>This forwards the request onto the server and collect its response into an instance of <i>ResponseBuilder</i> where then can be parsed into <i>Item</i>-s or parsed as <i>utf-8</i> encoded text content</p>
    </blockquote>
    <dd>
    </dd>

    <br>

    <dt>
    <p>download(<i>&self</i>, dest_prefix) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <blockquote>
    <p>Use this when there is no need to cache the entire response, this will save each response buffer directly to a local file</p>
    </blockquote>
    <dd>
    <p>dest_prefix: <i>&str</i>, The root folder for which the file will be saved in, subsequent path to file including file name will be taken from its path on the server</p>
    </dd>

    <br>

    <dt>
    <p>download_as(<i>&self</i>, dir_prefix, file_name) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <blockquote>
    <p>Use this when there is no need to cache the entire response, this will save each response buffer directly to a local file</p>
    </blockquote>
    <dd>
    <p>dir_prefix: <i>&str</i>, The path to folder for which the file will be save in</p>
    <p>file_name: <i>&str</i>, The file name for which the file will be saved as</p>
    </dd>

    </dl>
    </dd>
</dl>

<br>

- <dl><dt><p><strong>gopher::io::ResponseBuilder</strong></p></dt>
    <dd>
    <dl>
    <dt>
    <p>as_items(<i>&self</i>) &rarr; <i>Result&lt;Vec&lt;Item&gt;&gt;</i></p>
    </dt>
    <blockquote>
    <p>Parse server response into an array of <i>Item</i>-s</p>
    </blockquote>
    <dd>
    </dd>

    <br>

    <dt>
    <p>save_to_file(<i>&mut self</i>, dest_prefix) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <dd>
    <p>dest_prefix: <i>&str</i>, The root folder for which the file will be saved in, subsequent path to file including file name will be taken from its path on the server</p>
    </dd>

    <br>

    <dt>
    <p>save_as_file(<i>&mut self</i>, dir_prefix, file_name) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <dd>
    <p>dir_prefix: <i>&str</i>, The path to folder for which the file will be save in</p>
    <p>file_name: <i>&str</i>, The file name for which the file will be saved as</p>
    </dd>

    <br>

    <dt>
    <p>save_to_txt(<i>&mut self</i>, dest_prefix) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <blockquote>
    <p>Similar to save_to_file, this will treat the server response as a text response and save parsed content to a text file</p>
    </blockquote>
    <dd>
    <p>dest_prefix: <i>&str</i>, The root folder for which the file will be saved in, subsequent path to file including file name will be taken from its path on the server</p>
    </dd>

    <br>

    <dt>
    <p>save_as_txt(<i>&mut self</i>, dir_prefix, file_name) &rarr; <i>Result&lt;u64&gt;</i></p>
    </dt>
    <blockquote>
    <p>Similar to save_as_file, this will treat the server response as a text response and save parsed content to a text file</p>
    </blockquote>
    <dd>
    <p>dir_prefix: <i>&str</i>, The path to folder for which the file will be save in</p>
    <p>file_name: <i>&str</i>, The file name for which the file will be saved as</p>
    </dd>

    </dl>
    </dd>
</dl>

<br>

- <dl><dt><p><strong>gopher::types::Referer</strong></p></dt>
    <dd>
    <p>port: <i>u16</i></p>
    <p>path: <i>String</i></p>
    <p>domain: <i>String</i></p>
    </dd>
</dl>

<br>

- <dl><dt><p><strong>gopher::types::Item</strong></p></dt>
    <dd>
    <dl>
    <dt>
    <p>INFO</p>
    </dt>
    <blockquote>
    <p>Info or error items with first char (tag) from ' i ' | ' 3 '</p>
    </blockquote>
    <dd>
    <p>tag: <i>char</i></p>
    <p>from: <i>Referer</i></p>
    <p>message: <i>String</i></p>
    <p>domain: <i>String</i></p>
    <p>port: <i>u16</i></p>
    </dd>

    <br>

    <dt>
    <p>DATA</p>
    </dt>
    <blockquote>
    <p>Directory or binary items with first char (tag) from ' 0 ' | ' 1 ' | ' 4 ' | ' 5 ' | ' 7 ' | ' 9 ' | ' g ' | ' I ' | ' s '</p>
    </blockquote>
    <dd>
    <p>tag: <i>char</i></p>
    <p>size: <i>u64</i></p>
    <p>caption: <i>String</i></p>
    <p>referer: <i>Referer</i></p>
    <p>location: <i>String</i></p>
    <p>domain: <i>String</i></p>
    <p>port: <i>u16</i></p>
    </dd>

    <br>

    <dt>
    <p>UNKNOWN</p>
    </dt>
    <dd>
    <p>from: <i>Referer</i></p>
    <p>unparsed: <i>String</i></p>
    </dd>

    </dl>
    </dd>
</dl>

</body>