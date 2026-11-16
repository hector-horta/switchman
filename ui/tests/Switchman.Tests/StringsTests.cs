using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml.Linq;
using Xunit;

namespace Switchman.Tests
{
    /// <summary>
    /// Tests for string resource localization functionality.
    /// </summary>
    public sealed partial class StringsTests
    {
        private static string ResourcesDir()
        {
            var current = new DirectoryInfo(AppContext.BaseDirectory);
            while (current is not null && !File.Exists(Path.Combine(current.FullName, "Switchman.slnx")))
            {
                current = current.Parent;
            }
            return current is null
                ? throw new InvalidOperationException("Switchman.slnx not found")
                : Path.Combine(current.FullName, "src", "Switchman.Core", "Resources");
        }

        private static Dictionary<string, string> Load(string file)
        {
            var doc = XDocument.Load(Path.Combine(ResourcesDir(), file));
            return doc.Root!.Elements("data").ToDictionary(
                e => (string?)e.Attribute("name") ?? "",
                e => (string?)e.Element("value") ?? "");
        }

        [GeneratedRegex(@"\{[a-z0-9]+\}")]
        private static partial Regex Placeholder();

        [Fact]
        public void SpanishHasTheSameKeysAsEnglish()
        {
            var en = Load("Strings.resx");
            var es = Load("Strings.es.resx");

            Assert.Equal(en.Keys.Order(StringComparer.Ordinal), es.Keys.Order(StringComparer.Ordinal));
        }

        [Fact]
        public void PlaceholdersMatchInBothLanguages()
        {
            var en = Load("Strings.resx");
            var es = Load("Strings.es.resx");

            foreach (var key in en.Keys)
            {
                var enMatches = Placeholder().Matches(en[key])
                    .Select(m => m.Value)
                    .Order(StringComparer.Ordinal)
                    .ToList();

                var esMatches = Placeholder().Matches(es[key])
                    .Select(m => m.Value)
                    .Order(StringComparer.Ordinal)
                    .ToList();

                Assert.True(enMatches.SequenceEqual(esMatches), $"Mismatch in placeholders for key: {key}");
            }
        }

        [Fact]
        public void SpanishCultureLoadsSpanishText()
        {
            var originalCulture = CultureInfo.CurrentUICulture;

            try
            {
                CultureInfo.CurrentUICulture = new CultureInfo("es-CL");
                Assert.Equal("Guardar", Strings.Save);

                CultureInfo.CurrentUICulture = new CultureInfo("en-US");
                Assert.Equal("Save", Strings.Save);
            }
            finally
            {
                CultureInfo.CurrentUICulture = originalCulture;
            }
        }
    }
}
