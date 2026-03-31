class Repstruc < Formula
  include Language::Python::Virtualenv

  desc "Generate and maintain README structure blocks for every folder in a repository"
  homepage "https://github.com/owenarink/repstruc"
  url "https://github.com/owenarink/repstruc/archive/refs/tags/v0.2.1.tar.gz"
  sha256 "f289708ced0cf263905593f08837f93f4ea2bff3e64f579015f729e7f47b2ed0"
  license "MIT"

  depends_on "python@3.12"

  def install
    virtualenv_install_with_resources
  end

  test do
    mkdir "demo" do
      system bin/"repstruc", "."
      assert_predicate testpath/"demo/README.md", :exist?
    end
  end
end
