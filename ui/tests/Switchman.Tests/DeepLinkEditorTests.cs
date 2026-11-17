using Xunit;
using Switchman.Core.Models;
using Switchman.Core.ViewModels;
using Switchman.Core.Resources;

namespace Switchman.Tests
{
    public sealed class DeepLinkEditorTests
    {
        private readonly FakeCli _cli = new();

        private DeepLinkEditor EditorFor(DeepLinkItem link) => new(_cli, link, TimeSpan.Zero);

        [Fact]
        public void PickingASchemeFillsNameAndTemplate()
        {
            var link = new DeepLinkItem { Id = "dl1" };
            using var editor = EditorFor(link);
            link.Scheme = "steam";
            Assert.Equal("steam", link.Name);
            Assert.Equal("steam://{url}", link.Template);

            link.Scheme = "valve";
            Assert.Equal("valve", link.Name);
            Assert.Equal("valve://{url}", link.Template);
        }

        [Fact]
        public void ChangingTheSchemeKeepsTheRestOfTheTemplate()
        {
            var link = new DeepLinkItem { Id = "dl1", Name = "Steam", Scheme = "steam", Template = "steam://openurl/{url}" };
            using var editor = EditorFor(link);
            link.Scheme = "valve";
            Assert.Equal("valve://openurl/{url}", link.Template);
            Assert.Equal("Steam", link.Name);
        }

        [Fact]
        public void ExampleStartsWithTheFirstDomain()
        {
            var link = new DeepLinkItem { Id = "dl1", DomainsText = "store.steampowered.com" };
            using var editor = EditorFor(link);
            Assert.Equal("https://store.steampowered.com/", editor.ExampleUrl);
        }

        [Fact]
        public async Task IssuesShowNextToTheirField()
        {
            _cli.CheckReply = FakeCli.Ok(new DeepLinkCheck(
                new[] { new DeepLinkIssue("scheme", "scheme_invalid"), new DeepLinkIssue("template", "template_scheme") },
                true,
                new DeepLinkPreview(true, true, "x://y")
            ));

            var link = new DeepLinkItem { Id = "dl1", Name = "A", Scheme = "Bad" };
            using var editor = EditorFor(link);
            await editor.RefreshAsync(CancellationToken.None);

            Assert.Equal("", editor.NameError);
            Assert.Equal(Strings.DlSchemeInvalid, editor.SchemeError);
            Assert.Equal(Localized.Format(Strings.DlTemplateScheme, "bad"), editor.TemplateError);
            Assert.Equal("", editor.PreviewUri);
            Assert.False(editor.TryCommand.CanExecute(null));
        }

        [Fact]
        public async Task PreviewShowsTheUriAndTryOpensIt()
        {
            _cli.CheckReply = FakeCli.Ok(new DeepLinkCheck([], true, new DeepLinkPreview(true, true, "steam://openurl/https://a.com/")));

            var link = new DeepLinkItem { Id = "dl1", Name = "Steam", Scheme = "steam", DomainsText = "a.com", Template = "steam://openurl/{url}" };
            using var editor = EditorFor(link);
            await editor.RefreshAsync(CancellationToken.None);

            Assert.Equal("steam://openurl/https://a.com/", editor.PreviewUri);
            Assert.Equal("", editor.PreviewNote);

            await editor.TryCommand.ExecuteAsync(null);
            Assert.Contains("open-uri steam://openurl/https://a.com/", _cli.Calls);
        }

        [Fact]
        public async Task ExampleFromAnotherDomainWarns()
        {
            _cli.CheckReply = FakeCli.Ok(new DeepLinkCheck([], true, new DeepLinkPreview(true, false, "steam://x")));

            var link = new DeepLinkItem { Id = "dl1", Name = "Steam", Scheme = "steam", DomainsText = "a.com", Template = "steam://{url}" };
            using var editor = EditorFor(link);
            await editor.RefreshAsync(CancellationToken.None);

            Assert.Equal(Strings.DlExampleNoMatch, editor.PreviewNote);
        }
    }
}
