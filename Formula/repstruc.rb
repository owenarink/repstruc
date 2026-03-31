class Repstruc < Formula
  include Language::Python::Virtualenv

  desc "Generate and maintain README structure blocks for every folder in a repository"
  homepage "https://github.com/owenarink/repstruc"
  url "https://github.com/owenarink/repstruc/archive/refs/tags/v0.2.2.tar.gz"
  sha256 "bc7ff02122ef864b0788393f679237ccb9dd85ad476664b74219d1fb041e3802"
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
