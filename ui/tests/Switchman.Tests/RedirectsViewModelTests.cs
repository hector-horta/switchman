using Xunit;
using Switchman.Core.Models;
using Switchman.Core.Services;
using Switchman.Core.ViewModels;

namespace Switchman.Tests
{
    public sealed class RedirectsViewModelTests
    {
        private static readonly LinkConfig WithRedirects = new("edge", true, [], [], [], Redirects: [new Redirect("rd1", "reddit.com/mobile/download", "https://www.reddit.com/")]);

        private readonly FakeCli _cli = new();

        private async Task<RedirectsViewModel> LoadedAsync()
        {
            _cli.GetConfigReplies.Enqueue(FakeCli.Ok(new VersionedConfig("rev1", WithRedirects)));
            var vm = new RedirectsViewModel(new ConfigSession(_cli, new FakeDialogs()));
            await vm.LoadCommand.ExecuteAsync(null);
            return vm;
        }

        [Fact]
        public async Task LoadFillsTheList()
        {
            var vm = await LoadedAsync();
            var item = Assert.Single(vm.Redirects);
            Assert.Equal("reddit.com/mobile/download", item.From);
            Assert.Equal("https://www.reddit.com/", item.To);
            Assert.False(vm.HasSelection);
        }

        [Fact]
        public async Task AddSelectsANewRedirectWithAFreeId()
        {
            var vm = await LoadedAsync();
            vm.AddRedirectCommand.Execute(null);
            Assert.Equal(2, vm.Redirects.Count);
            Assert.Equal("rd2", vm.SelectedRedirect!.Id);
            Assert.Equal(Strings.RedirectsNewTitle, vm.SelectedRedirect.Title);
        }

        [Fact]
        public async Task SaveSendsTrimmedRedirects()
        {
            var vm = await LoadedAsync();
            vm.AddRedirectCommand.Execute(null);
            vm.SelectedRedirect!.From = " www.whatsapp.com/ ";
            vm.SelectedRedirect.To = " whatsapp:// ";
            _cli.SetConfigReplies.Enqueue(FakeCli.Ok(new SaveResult("rev2")));
            await vm.SaveCommand.ExecuteAsync(null);

            var saved = _cli.LastSaved!.Redirects!;
            Assert.Equal(2, saved.Count);
            Assert.Equal(new Redirect("rd2", "www.whatsapp.com/", "whatsapp://"), saved[1]);
            Assert.Equal(Strings.SaveSaved, vm.Message);
        }

        [Fact]
        public async Task RemoveDropsTheRedirectAndTheSelection()
        {
            var vm = await LoadedAsync();
            var item = vm.Redirects[0];
            vm.SelectedRedirect = item;
            vm.RemoveRedirectCommand.Execute(item);

            Assert.Empty(vm.Redirects);
            Assert.Null(vm.SelectedRedirect);
        }
    }
}
