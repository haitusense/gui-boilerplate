
try{
  await WritePipe("wuzeiNamedPipe", $$"""{ "type" : "status", "payload": true }""");
  for(var i=0;i<5;i++){
    System.Threading.Thread.Sleep(1000);
    Console.WriteLine(i);
  }
  var json = $$"""{ "url" : "null", "date" : "null", "title" : "null", "subtitle" : "null" }""";
  await WritePipe(
    "wuzeiNamedPipe", 
    $$"""{ "type" : "postjson", "payload": {{json}} } """
  );
} catch(Exception e) {
  Console.WriteLine(e);
} finally {
  await WritePipe("wuzeiNamedPipe", $$"""{ "type" : "status", "payload": false }""");
}


static async Task<string> WritePipe(string addr, string val){
  string temp = "Err";
  try{ 
    using var pipe  = new System.IO.Pipes.NamedPipeClientStream(".", addr, System.IO.Pipes.PipeDirection.InOut, System.IO.Pipes.PipeOptions.Asynchronous);
    await pipe.ConnectAsync();
    using var sw = new StreamWriter(pipe);
    using var sr = new StreamReader(pipe);
    await sw.WriteLineAsync(val);
    pipe.WaitForPipeDrain();
    await sw.FlushAsync();
    temp = await sr.ReadLineAsync();
    sw.Close();
    await sw.DisposeAsync();
  } catch ( IOException ) { Console.WriteLine("DisConnect");
  } catch ( Exception e ) { Console.WriteLine(e.ToString()); }
  return temp;
}
